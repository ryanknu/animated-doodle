use crate::errors::ChatError;
use aws_sdk_dynamodb::{
    error::PutItemError,
    model::{AttributeValue, KeysAndAttributes},
    output::PutItemOutput,
    types::SdkError,
};
use std::collections::HashMap;

// Messages

pub async fn get_message_by_id(
    dynamodb: &aws_sdk_dynamodb::Client,
    room_id: &str,
    message_id: &str,
) -> Result<HashMap<String, AttributeValue>, ChatError> {
    let output = dynamodb
        .get_item()
        .table_name("messages")
        .key("room_id", AttributeValue::N(room_id.to_owned()))
        .key("sort", AttributeValue::S(format!("message.{}", message_id)))
        .projection_expression("room_id,sort,sender_id,sender_name,message")
        .send()
        .await?;

    Ok(output.item.ok_or_else(query_error)?)
}

pub async fn get_messages(
    dynamodb: &aws_sdk_dynamodb::Client,
    room_id: &str,
    limit: u8,
) -> Result<Vec<HashMap<String, AttributeValue>>, ChatError> {
    let output = dynamodb
        .query()
        .table_name("messages")
        .key_condition_expression("room_id = :r AND begins_with(sort, :m)")
        .expression_attribute_values(":r", AttributeValue::N(room_id.to_owned()))
        .expression_attribute_values(":m", AttributeValue::S("message.".into()))
        .scan_index_forward(false)
        .limit(limit.into())
        .send()
        .await?;

    Ok(output.items.ok_or_else(query_error)?)
}

pub async fn post_message(
    dynamodb: &aws_sdk_dynamodb::Client,
    room_id: &str,
    message: &str,
    sender_id: &str,
    sender_name: &str,
    date_time: &str,
) -> Result<PutItemOutput, ChatError> {
    Ok(dynamodb
        .put_item()
        .table_name("messages")
        .item("room_id", AttributeValue::N(room_id.to_owned()))
        .item("sort", AttributeValue::S(format!("message.{}", date_time)))
        .item("sender_id", AttributeValue::N(sender_id.to_owned()))
        .item("sender_name", AttributeValue::S(sender_name.to_owned()))
        .item("message", AttributeValue::S(message.to_owned()))
        .send()
        .await?)
}

// Rooms

/// Takes a string of CSV entries and an input. Removes the input from the
/// middle of the CSV string and any preceding comma, and then appends it to
/// the end of a new allocated string.
/// For example:
/// ```
/// assert_eq!("a", bump_csv("a", "a"));
/// assert_eq!("a", bump_csv("", "a"))
/// assert_eq!("b,c,d,a", bump_csv("a,b,c,d", "a"));
/// assert_eq!("a,c,d,b", bump_csv("a,b,c,d", "b"));
/// assert_eq!("a,b,d,c", bump_csv("a,b,c,d", "c"));
/// assert_eq!("a,b,c,d", bump_csv("a,b,c,d", "d"));
/// ```
/// This example (all using size-1 strings) is not all that contrived because
/// in practice the string will likely contain all elements of length 32.
fn bump_csv(csv_string: &str, input: &str) -> String {
    match csv_string.find(input) {
        None => {
            if csv_string.is_empty() {
                input.to_owned()
            } else {
                format!("{},{}", csv_string, input)
            }
        }
        Some(i) => {
            let has_preceding_comma = i > 0 && csv_string[i - 1..i].eq(",");
            let i = if has_preceding_comma { i - 1 } else { i };
            let l = if has_preceding_comma {
                input.len() + 1
            } else {
                input.len()
            };
            format!("{}{},{}", &csv_string[0..i], &csv_string[i + l..], input)
        }
    }
}

pub async fn bump_room(
    dynamodb: &aws_sdk_dynamodb::Client,
    room_id: &str,
) -> Result<PutItemOutput, ChatError> {
    let room_ids_csv = get_active_rooms_scalar(dynamodb).await?;
    let room_ids_csv = bump_csv(&room_ids_csv, room_id);

    Ok(dynamodb
        .put_item()
        .table_name("messages")
        .item("room_id", AttributeValue::N("1".into()))
        .item("sort", AttributeValue::S("active_rooms".into()))
        .item("room_ids", AttributeValue::S(room_ids_csv))
        .send()
        .await?)
}

pub async fn create_room(
    dynamodb: &aws_sdk_dynamodb::Client,
    room_id: &str,
    room_name: &str,
) -> Result<PutItemOutput, SdkError<PutItemError>> {
    Ok(dynamodb
        .put_item()
        .table_name("messages")
        .item("room_id", AttributeValue::N(room_id.to_owned()))
        .item("sort", AttributeValue::S("room".into()))
        .item("name", AttributeValue::S(room_name.to_owned()))
        .send()
        .await?)
}

async fn get_active_rooms_scalar(dynamodb: &aws_sdk_dynamodb::Client) -> Result<String, ChatError> {
    let output = dynamodb
        .get_item()
        .table_name("messages")
        .key("room_id", AttributeValue::N("1".into()))
        .key("sort", AttributeValue::S("active_rooms".into()))
        .projection_expression("room_ids")
        .send()
        .await?;

    match output.item {
        None => Ok(String::new()),
        Some(item) => Ok(item["room_ids"].as_s()?.clone()),
    }
}

pub async fn get_active_rooms(
    dynamodb: &aws_sdk_dynamodb::Client,
) -> Result<Vec<HashMap<String, AttributeValue>>, ChatError> {
    let room_ids_csv = get_active_rooms_scalar(dynamodb).await?;
    println!("{:?}", &room_ids_csv);

    let mut room_ids: Vec<HashMap<String, AttributeValue>> = Vec::new();
    for room_id in room_ids_csv.split(",") {
        // Split will always have at least one element. Doing the check here
        // also makes the code to tolerant to leading and trialing commas, I
        // guess you could argue is a bad thing (no preconditions = bad).
        // Tolerating bad data means bugs don't get fixed.
        if room_id.is_empty() {
            continue;
        }
        let mut primary_key: HashMap<String, AttributeValue> = HashMap::new();
        primary_key.insert("room_id".into(), AttributeValue::N(room_id.to_owned()));
        primary_key.insert("sort".into(), AttributeValue::S("room".into()));
        room_ids.push(primary_key);
    }

    println!("{:?}", &room_ids);

    if room_ids.is_empty() {
        return Ok(vec![]);
    }

    // TODO: sort the list; that's a requirement

    let output = dynamodb
        .batch_get_item()
        .request_items(
            "messages",
            KeysAndAttributes::builder()
                .set_keys(Some(room_ids))
                .projection_expression("room_id,#n")
                .expression_attribute_names("#n", "name")
                .build(),
        )
        .send()
        .await?;

    println!("{:?}", output.responses);

    Ok(output.responses.ok_or_else(query_error)?["messages"].clone())
}

// Users

pub async fn create_user(
    dynamodb: &aws_sdk_dynamodb::Client,
    user_id: &str,
    user_name: &str,
) -> Result<PutItemOutput, SdkError<PutItemError>> {
    Ok(dynamodb
        .put_item()
        .table_name("users")
        .item("user_id", AttributeValue::N(user_id.to_owned()))
        .item("name", AttributeValue::S(user_name.to_owned()))
        .send()
        .await?)
}

pub async fn get_user_by_id(
    dynamodb: &aws_sdk_dynamodb::Client,
    user_id: &str,
) -> Result<HashMap<String, AttributeValue>, ChatError> {
    let output = dynamodb
        .get_item()
        .table_name("users")
        .key("user_id", AttributeValue::N(user_id.to_owned()))
        .projection_expression("user_id,#n")
        .expression_attribute_names("#n", "name")
        .send()
        .await?;

    Ok(output
        .item
        .ok_or_else(|| ChatError::new(None, "User does not exist".into()))?
        .clone())
}

pub async fn get_user_by_name(
    dynamodb: &aws_sdk_dynamodb::Client,
    user_name: &str,
) -> Result<HashMap<String, AttributeValue>, ChatError> {
    let output = dynamodb
        .query()
        .table_name("users")
        .index_name("name-index")
        .projection_expression("user_id,name")
        .key_condition_expression("#n=:n")
        .expression_attribute_names("#n", "name")
        .expression_attribute_values(":n", AttributeValue::S(user_name.to_owned()))
        .send()
        .await?;

    if output.count() != 1 {
        return Err(ChatError::new(None, "Result DNE or is ambiguous".into()));
    }

    Ok(output
        .items
        .ok_or_else(query_error)?
        .first()
        .ok_or_else(query_error)?
        .clone())
}

pub async fn user_exists(dynamodb: &aws_sdk_dynamodb::Client, user_name: &str) -> bool {
    matches!(get_user_by_name(dynamodb, user_name).await, Ok(_))
}

fn query_error() -> ChatError {
    ChatError::new(None, "Query error".into())
}
