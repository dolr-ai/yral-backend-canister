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
        struct NotificationV2 {
            notifications: Vec<NotificationDataV2>,
            last_viewed: Option<SystemTime>,
        }
        
        #[derive(Deserialize)]
        struct NotificationDataV2 {
            notification_id: u64,
            payload: NotificationTypeV2,
            created_at: SystemTime,
        }
        
        #[derive(Deserialize)]
        enum NotificationTypeV2 {
            Liked(LikedPayloadV2),
            VideoUpload(VideoUploadPayloadV2),
        }
        
        #[derive(Deserialize)]
        struct LikedPayloadV2 {
            by_user_principal: Principal,
            post_id: u64,
        }
        
        #[derive(Deserialize)]
        struct VideoUploadPayloadV2 {
            #[serde(alias = "video_id")]
            video_uid: u64,
        }
        
        if let Ok(v2) = de::from_reader::<NotificationV2, _>(bytes.as_ref()) {
            return Notification {
                notifications: v2.notifications.into_iter().map(|data| {
                    NotificationData {
                        notification_id: data.notification_id,
                        payload: match data.payload {
                            NotificationTypeV2::Liked(p) => NotificationType::Liked(LikedPayload {
                                by_user_principal: p.by_user_principal,
                                post_id: p.post_id.to_string(),
                            }),
                            NotificationTypeV2::VideoUpload(p) => NotificationType::VideoUpload(VideoUploadPayload {
                                video_uid: p.video_uid.to_string(),
                            }),
                        },
                        created_at: data.created_at,
                    }
                }).collect(),
                last_viewed: v2.last_viewed,
            };
        }
        
        // Try v1 format: tuple struct with Vec and read field
        #[derive(Deserialize)]
        struct NotificationV1(Vec<NotificationDataV1>);
        
        #[derive(Deserialize)]
        struct NotificationDataV1 {
            notification_id: u64,
            payload: NotificationTypeV1,
            read: bool,
            created_at: SystemTime,
        }
        
        #[derive(Deserialize)]
        enum NotificationTypeV1 {
            Liked(LikedPayloadV1),
            VideoUpload(VideoUploadPayloadV1),
        }
        
        #[derive(Deserialize)]
        struct LikedPayloadV1 {
            by_user_principal: Principal,
            post_id: u64,
        }
        
        #[derive(Deserialize)]
        struct VideoUploadPayloadV1 {
            #[serde(alias = "video_id")]
            video_uid: u64,
        }
        
        let v1: NotificationV1 = de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize notification from stable storage");
        
        Notification {
            notifications: v1.0.into_iter().map(|data| {
                NotificationData {
                    notification_id: data.notification_id,
                    payload: match data.payload {
                        NotificationTypeV1::Liked(p) => NotificationType::Liked(LikedPayload {
                            by_user_principal: p.by_user_principal,
                            post_id: p.post_id.to_string(),
                        }),
                        NotificationTypeV1::VideoUpload(p) => NotificationType::VideoUpload(VideoUploadPayload {
                            video_uid: p.video_uid.to_string(),
                        }),
                    },
                    created_at: data.created_at,
                }
            }).collect(),
            last_viewed: None, // v1 didn't have last_viewed
        }
    }
}