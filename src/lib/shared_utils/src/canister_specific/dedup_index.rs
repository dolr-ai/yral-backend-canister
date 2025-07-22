use std::{
    collections::BTreeSet,
    ops::{Deref, DerefMut},
    time::SystemTime,
};

use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};

pub type VideoId = String;
pub type VideoHash = String;

pub const LISTING_SIZE_LIMIT_EXCLUSIVE: usize = 50;

#[derive(Debug, candid::CandidType, Deserialize)]
pub enum ListError {
    PageOutOfRange,
    SizeNotAllowed,
    WillOverflow,
}

#[derive(Debug, candid::CandidType, Serialize, Deserialize)]
pub struct ListArgs {
    pub page: usize,
    pub size: usize,
}

pub type Video = (VideoId, SystemTime);
#[derive(Clone, Debug, Serialize, Deserialize, CandidType, Default)]
pub struct Videos(pub BTreeSet<Video>);

impl Deref for Videos {
    type Target = BTreeSet<Video>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Videos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Storable for Videos {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();

        std::borrow::Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let value: Self = ciborium::de::from_reader(bytes.as_ref()).unwrap();
        value
    }
}
