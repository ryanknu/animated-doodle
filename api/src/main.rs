use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Endpoint, Region};
use axum::{
    extract,
    routing::{get, post, put},
    Json, Router,
};
use errors::ChatError;
use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Method,
};
use hyper::Uri;
use models::*;
use tower_http::cors::{CorsLayer, Origin};

mod db;
mod errors;
mod init;
mod models;

/// Last minute todos:
/// - document it
///   - document tables in init.rs
/// - see if we can add middleware
/// - look for unused casts in errors.rs
/// - double check that all id's are N and everything else is S

#[tokio::main]
async fn main() {
    let dynamodb = dynamodb_client().await;
    init::init(&dynamodb).await;

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT])
        .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE])
        .allow_origin(Origin::exact(
            std::env::var("ACCESS_CONTROL_ALLOW_ORIGIN")
                .unwrap()
                .parse()
                .unwrap(),
        ));

    let app = Router::new()
        .route("/", get(hateos))
        .route("/sign-up", post(sign_up))
        .route("/sign-in", post(sign_in))
        .route("/users/:user_id", get(get_user))
        .route(
            "/rooms/:room_id/messages/:message_id",
            get(get_message_by_id),
        )
        .route("/rooms/:room_id/messages", get(get_messages))
        .route("/rooms/:room_id/messages", put(put_message))
        .route("/rooms", put(put_room))
        .route("/rooms", get(get_rooms))
        .route("/status", get(|| async { "OK" }))
        .layer(cors);

    let port = port();
    axum::Server::bind(&format!("0.0.0.0:{}", port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// DynamoDB returns data as a HashMap<String, AttributeValue>. This can be
/// tiresome to parse and handle all situations, so we define a macro. The
/// following statements are equivalent:
/// ```
/// S!(map, "key")
/// map["key"].to_s()?
/// ```
/// However, we have additional error handling added via the macro that raises
/// an error when an index does not exist.
/// Generally speaking, macros aren't the clearest thing in the world to use,
/// but for situations like this where you want to generate a custom error
/// message based on inputs, I think there's no better tool.
macro_rules! S {
    ($map:ident,$key:literal) => {
        $map.get($key)
            .ok_or_else(|| {
                let map = stringify!($map);
                ChatError::new(
                    Some(format!("Could not index {} by {}", map, $key)),
                    "Internal server error".into(),
                )
            })?
            .as_s()?
    };
}

/// See explanation for S!, however, this works with attribute type N.
macro_rules! N {
    ($map:ident,$key:literal) => {
        $map.get($key)
            .ok_or_else(|| {
                let map = stringify!($map);
                ChatError::new(
                    Some(format!("Could not index {} by {}", map, $key)),
                    "Internal server error".into(),
                )
            })?
            .as_n()?
    };
}

/// Retrieves the DynamoDB client. You can wrap this into a `static OnceCell`
/// to avoid having to call and pass this around all of the time, but, this
/// isn't a production-grade application, and this makes things a little easier
/// to understand where things come from as you can trace all HTTP handlers
/// back to where their data comes from. Plus, doing this "inefficient" thing
/// adds less overhead than starting up the PHP interpreter, so you know, all
/// things in moderation.
/// I'd be more concrened about if the AWS SDK becomes unstable if you create
/// way more clients than expected. Their documentation does not provide best
/// practices on client handling.
/// Lots of instantiating and passing around isn't great, but global static
/// variables are efficient and fast, but also "magic".
pub async fn dynamodb_client() -> aws_sdk_dynamodb::Client {
    let dynamodb_hostname = std::env::var("DB_HOSTNAME");
    let region = std::env::var("AWS_REGION").ok();
    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-east-1"));
    let config = aws_config::from_env().region(region_provider).load().await;
    let config = aws_sdk_dynamodb::config::Builder::from(&config);
    let config = match dynamodb_hostname {
        Ok(hostname) => config.endpoint_resolver(Endpoint::immutable(
            format!("http://{}:8000", hostname).parse::<Uri>().unwrap(),
        )),
        _ => config,
    };
    aws_sdk_dynamodb::Client::from_conf(config.build())
}

fn port() -> String {
    match std::env::var("PORT") {
        Ok(port) => port,
        _ => "80".into(),
    }
}

/// Gets the hostname of the current server. This could be expanded to look at
/// the `Host` HTTP header if we want to, but, this will work for today. It's
/// pretty normal to have to override the hostname the server thinks it's at,
/// with docker and proxies being very common places servers can get pretty
/// confused about their "hostname" and the public hostname that can makes
/// them publicly routable.
/// Port is not appended here for the same reason, often times, the port the
/// app is listening on is not the port that is publicly routable.
fn hostname() -> String {
    match std::env::var("HOSTNAME") {
        Ok(hostname) => hostname,
        _ => "api".into(),
    }
}

/// Generates a UUID for new objects. This is good enough for now. Ideally
/// what you want for DynamoDB is something that generates a random value
/// between 0 and 99..99 (38 9's). Left-substring is a cheap way to get this
/// but this means that multiple random outputs can have the same uuid()
fn uuid() -> String {
    fastrand::u128(u128::MIN..u128::MAX).to_string()[..38].to_owned()
}

async fn sign_up(
    extract::Json(name_request): extract::Json<NameRequest>,
) -> Result<Json<User>, ChatError> {
    let dynamodb = dynamodb_client().await;
    let hostname = hostname();

    if name_request.name.is_empty() {
        return Err(ChatError::new(None, "Name is empty".into()));
    }

    if db::user_exists(&dynamodb, &name_request.name).await {
        Err(ChatError::new(None, "Name already registered".into()))
    } else {
        let id = uuid();
        db::create_user(&dynamodb, &id, &name_request.name).await?;
        Ok(Json(Object::user(
            &format!("http://{}/users/{}", hostname, &id),
            &name_request.name,
        )))
    }
}

async fn sign_in(
    extract::Json(name_request): extract::Json<NameRequest>,
) -> Result<Json<User>, ChatError> {
    let dynamodb = dynamodb_client().await;
    let hostname = hostname();

    if name_request.name.is_empty() {
        return Err(ChatError::new(None, "Name is empty".into()));
    }

    let user = db::get_user_by_name(&dynamodb, &name_request.name).await?;
    Ok(Json(Object::user(
        &format!("http://{}/users/{}", hostname, N!(user, "user_id")),
        S!(user, "name"),
    )))
}

async fn get_user(extract::Path(user_id): extract::Path<String>) -> Result<Json<User>, ChatError> {
    let dynamodb = dynamodb_client().await;
    let hostname = hostname();
    let slash_index = user_id
        .rfind('/')
        .ok_or_else(|| ChatError::new(None, "Invalid user_id".into()))?;

    let user = db::get_user_by_id(&dynamodb, &user_id[slash_index + 1..]).await?;
    Ok(Json(Object::user(
        &format!("http://{}/users/{}", hostname, N!(user, "user_id")),
        S!(user, "name"),
    )))
}

/// Retrieves a message by it's ID. This handler wasn't asked for in the
/// requirements but I found it necessary to add because otherwise the ID for
/// a message would be a URI to a 404, which seems uncool.
async fn get_message_by_id(
    extract::Path((room_id, message_id)): extract::Path<(String, String)>,
) -> Result<Json<Message>, ChatError> {
    let dynamodb = dynamodb_client().await;
    let hostname = hostname();

    let message = db::get_message_by_id(&dynamodb, &room_id, &message_id).await?;

    // Date time is part of the sort key. It is of the format `message.TIME`
    // so we simply do a substring from index 8. Dangerous? You betcha.
    // A better solution might be to confirm that length > 8 and that the
    // substring [8..] is formatted like a date.
    let date_time = &S!(message, "sort")[8..];

    Ok(Json(Object::message(
        &format!(
            "http://{}/rooms/{}/messages/{}",
            hostname,
            room_id,
            S!(message, "sort")
        ),
        date_time,
        S!(message, "sender_name"),
        S!(message, "message"),
        &format!("http://{}/rooms/{}", hostname, room_id),
        &format!("http://{}/users/{}", hostname, N!(message, "sender_id")),
    )))
}

/// Retrieves the latest messages in a room.
async fn get_messages(
    extract::Path(room_id): extract::Path<String>,
) -> Result<Json<Vec<Message>>, ChatError> {
    let dynamodb = dynamodb_client().await;
    let hostname = hostname();
    let mut messages = Vec::new();

    for message in db::get_messages(&dynamodb, &room_id, 50).await? {
        let date_time = &S!(message, "sort")[8..];
        messages.push(Object::message(
            &format!(
                "http://{}/rooms/{}/messages/{}",
                hostname,
                room_id,
                S!(message, "sort")
            ),
            date_time,
            S!(message, "sender_name"),
            S!(message, "message"),
            &format!("http://{}/rooms/{}", hostname, room_id),
            &format!("http://{}/users/{}", hostname, N!(message, "sender_id")),
        ))
    }

    Ok(Json(messages))
}

async fn put_message(
    extract::Path(room_id): extract::Path<String>,
    extract::Json(message_request): extract::Json<MessageRequest>,
) -> Result<Json<Message>, ChatError> {
    let dynamodb = dynamodb_client().await;
    let hostname = hostname();
    let id = uuid();
    let date_time = chrono::Utc::now().to_rfc3339();

    if message_request.message.is_empty() {
        return Err(ChatError::new(None, "Name is empty".into()));
    }

    // Message_request.sender_id is a URI so we just need to take off the last
    // index.
    let sender_id = message_request.sender_id;
    let slash_index = sender_id
        .rfind('/')
        .ok_or_else(|| ChatError::new(None, "Invalid sender_id".into()))?;
    let sender_id = &sender_id[slash_index + 1..];

    // Look up user
    let user = db::get_user_by_id(&dynamodb, sender_id).await?;
    let user_name = S!(user, "name");

    // Insert message
    db::post_message(
        &dynamodb,
        &room_id,
        &message_request.message,
        &sender_id,
        &user_name,
        &date_time,
    )
    .await?;

    // Bump room to top of room listing
    db::bump_room(&dynamodb, &room_id).await?;

    Ok(Json(Object::message(
        &format!("http://{}/rooms/{}/messages/{}", hostname, room_id, id),
        &date_time,
        user_name,
        &message_request.message,
        &format!("http://{}/rooms/{}", hostname, room_id),
        &format!("http://{}/users/{}", hostname, sender_id),
    )))
}

async fn get_rooms() -> Result<Json<Vec<Room>>, ChatError> {
    let dynamodb = dynamodb_client().await;
    let hostname = hostname();
    let mut r = Vec::new();

    for room in db::get_active_rooms(&dynamodb).await? {
        println!("{:?}", &room);
        r.push(Object::room(
            &format!("http://{}/rooms/{}", hostname, N!(room, "room_id")),
            S!(room, "name"),
            &format!("http://{}/rooms/{}/messages", hostname, N!(room, "room_id")),
        ));
    }

    Ok(Json(r))
}

async fn put_room(
    extract::Json(name_request): extract::Json<NameRequest>,
) -> Result<Json<Room>, ChatError> {
    let dynamodb = dynamodb_client().await;
    let hostname = hostname();
    let id = uuid();
    // todo: check if the room already exists
    db::create_room(&dynamodb, &id, &name_request.name).await?;
    db::bump_room(&dynamodb, &id).await?;
    Ok(Json(Object::room(
        &format!("http://{}/rooms/{}", hostname, id),
        &name_request.name,
        &format!("http://{}/rooms/{}/messages", hostname, id),
    )))
}

async fn hateos() -> String {
    format!(
        r#"{{
    "id": "http://{x}/",
    "properties": {{
        "rooms": "http://{x}/rooms,
        "sign_in": "http://{x}/sign-in",
        "sign_up": "http://{x}/sign-up",
        "status": "http://{x}/status"
    }}
}}"#,
        x = hostname()
    )
}
