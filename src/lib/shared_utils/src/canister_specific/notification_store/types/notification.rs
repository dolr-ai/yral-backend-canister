use std::borrow::Cow;

use candid::{CandidType, Principal};
use ciborium::de;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Notification(pub Vec<NotificationData>);

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct NotificationData {
    pub notification_id: u64,
    pub payload: NotificationType,
    pub read: bool,
    pub created_at: u64,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct LikedPayload {
    pub by_user_principal: Principal,
    pub post_id: u64,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct VideoUploadPayload {
    #[serde(alias = "video_id")]
    pub video_uid: u64,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub enum NotificationType {
    Liked(LikedPayload),
    VideoUpload(VideoUploadPayload),
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
