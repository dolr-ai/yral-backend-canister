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
        if let Ok(notification) = de::from_reader::<Notification, _>(bytes.as_ref()) {
            return notification;
        }
        
        #[derive(Deserialize)]
        struct OldNotification {
            notifications: Vec<OldNotificationData>,
            last_viewed: Option<SystemTime>,
        }
        
        #[derive(Deserialize)]
        struct OldNotificationData {
            notification_id: u64,
            payload: OldNotificationType,
            created_at: SystemTime,
        }
        
        #[derive(Deserialize)]
        enum OldNotificationType {
            Liked(OldLikedPayload),
            VideoUpload(OldVideoUploadPayload),
        }
        
        #[derive(Deserialize)]
        struct OldLikedPayload {
            by_user_principal: Principal,
            post_id: u64,
        }
        
        #[derive(Deserialize)]
        struct OldVideoUploadPayload {
            #[serde(alias = "video_id")]
            video_uid: u64,
        }
        
        let old: OldNotification = de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize notification from stable storage");
        
        Notification {
            notifications: old.notifications.into_iter().map(|old_data| {
                NotificationData {
                    notification_id: old_data.notification_id,
                    payload: match old_data.payload {
                        OldNotificationType::Liked(p) => NotificationType::Liked(LikedPayload {
                            by_user_principal: p.by_user_principal,
                            post_id: p.post_id.to_string(),
                        }),
                        OldNotificationType::VideoUpload(p) => NotificationType::VideoUpload(VideoUploadPayload {
                            video_uid: p.video_uid.to_string(),
                        }),
                    },
                    created_at: old_data.created_at,
                }
            }).collect(),
            last_viewed: old.last_viewed,
        }
    }
}