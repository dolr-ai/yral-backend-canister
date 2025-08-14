use std::{borrow::Cow, time::SystemTime};

use candid::{CandidType, Principal};
use ciborium::de;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, CandidType, PartialEq)]
pub struct Notification {
    pub notifications: Vec<NotificationData>,
    pub last_viewed: Option<SystemTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug, CandidType, PartialEq)]
pub struct NotificationData {
    pub notification_id: u64,
    pub payload: NotificationType,
    pub created_at: std::time::SystemTime,
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub struct LikedPayload {
    pub by_user_principal: Principal,
    #[serde(deserialize_with = "string_or_number")]
    pub post_id: String,
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub struct VideoUploadPayload {
    #[serde(alias = "video_id", deserialize_with = "string_or_number")]
    pub video_uid: String,
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub enum NotificationType {
    Liked(LikedPayload),
    VideoUpload(VideoUploadPayload),
}

fn string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(u64),
    }

    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => Ok(s),
        StringOrNumber::Number(n) => Ok(n.to_string()),
    }
}

impl Storable for Notification {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let notification: Notification = de::from_reader(bytes.as_ref()).unwrap();
        notification
    }
}