use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Object<T>
where
    T: Serialize,
{
    id: String,
    properties: T,
}

#[derive(Deserialize)]
pub struct MessageRequest {
    pub message: String,
    pub sender_id: String,
}

pub type Message = Object<MessageProperties>;

#[derive(Serialize)]
pub struct MessageProperties {
    pub date_time: String,
    pub sender_name: String,
    pub message: String,
    pub room: String,
    pub user: String,
}

impl Object<MessageProperties> {
    pub fn message(
        id: &str,
        date_time: &str,
        sender_name: &str,
        message: &str,
        room: &str,
        user: &str,
    ) -> Self {
        Self {
            id: id.to_owned(),
            properties: MessageProperties {
                date_time: date_time.to_owned(),
                sender_name: sender_name.to_owned(),
                message: message.to_owned(),
                room: room.to_owned(),
                user: user.to_owned(),
            },
        }
    }
}

#[derive(Deserialize)]
pub struct NameRequest {
    pub name: String,
}

pub type Room = Object<RoomProperties>;

#[derive(Serialize)]
pub struct RoomProperties {
    pub name: String,
    pub messages: String,
}

impl Object<RoomProperties> {
    pub fn room(id: &str, name: &str, messages: &str) -> Self {
        Self {
            id: id.to_owned(),
            properties: RoomProperties {
                name: name.to_owned(),
                messages: messages.to_owned(),
            },
        }
    }
}

pub type User = Object<UserProperties>;

#[derive(Serialize)]
pub struct UserProperties {
    pub name: String,
}

impl Object<UserProperties> {
    pub fn user(id: &str, name: &str) -> Self {
        Self {
            id: id.to_owned(),
            properties: UserProperties {
                name: name.to_owned(),
            },
        }
    }
}
