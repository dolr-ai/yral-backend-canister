use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::{cmp::Ordering, time::SystemTime};

#[derive(Serialize, Deserialize, CandidType, Clone, Default, Debug, PartialEq, Eq, Hash, Copy)]
pub enum PostStatus {
    #[default]
    Uploaded,
    Transcoding,
    CheckingExplicitness,
    BannedForExplicitness,
    ReadyToView,
    BannedDueToUserReporting,
    Deleted,
}

impl From<PostStatus> for PostStatusV1 {
    fn from(value: PostStatus) -> Self {
        match value {
            PostStatus::Transcoding => PostStatusV1::Published,
            PostStatus::CheckingExplicitness => PostStatusV1::CheckingExplicitness,
            PostStatus::BannedForExplicitness => PostStatusV1::BannedForExplicitness,
            PostStatus::ReadyToView => PostStatusV1::Published,
            PostStatus::BannedDueToUserReporting => PostStatusV1::BannedDueToUserReporting,
            PostStatus::Deleted => PostStatusV1::Deleted,
            PostStatus::Uploaded => PostStatusV1::Published,
        }
    }
}

#[derive(Serialize, Deserialize, CandidType, Clone, Default, Debug, PartialEq, Eq, Hash, Copy)]
#[serde(from = "PostStatus")]
pub enum PostStatusV1 {
    #[default]
    Published,
    BannedForExplicitness,
    CheckingExplicitness,
    BannedDueToUserReporting,
    Deleted,
    Draft,
}

#[derive(Clone, CandidType, Deserialize, Debug, Serialize)]
pub struct PostScoreIndexItem {
    pub score: u64,
    pub post_id: u64,
    pub publisher_canister_id: Principal,
}

#[derive(Clone, CandidType, Deserialize, Debug, Serialize, PartialEq, Eq)]
pub struct PostScoreIndexItemV1 {
    pub score: u64,
    pub post_id: u64,
    pub publisher_canister_id: Principal,
    #[serde(default)]
    pub is_nsfw: bool,
    #[serde(default)]
    pub created_at: Option<SystemTime>,
    #[serde(default)]
    pub status: PostStatus,
}

// #[derive(Debug, PartialEq, Eq)]
// struct MyType {
//     id: u32,
//     score: u32,
// }

// impl Ord for MyType {
//     fn cmp(&self, other: &Self) -> Ordering {
//         // Compare by the `id` field for equality first
//         let id_cmp = self.id.cmp(&other.id);

//         if id_cmp != Ordering::Equal {
//             // If the `id` fields are equal, compare by the `score` field
//             self.score.cmp(&other.score)
//         } else {
//             // If the `id` fields are different, return the `id` comparison result
//             id_cmp
//         }
//     }
// }

// impl PartialOrd for MyType {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl PartialEq for MyType {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id
//     }
// }

// impl Eq for MyType {}

impl Ord for PostScoreIndexItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.publisher_canister_id.cmp(&self.publisher_canister_id) {
            Ordering::Equal => match other.post_id.cmp(&self.post_id) {
                Ordering::Equal => Ordering::Equal,
                _ => other.score.cmp(&self.score),
            },
            _ => other.score.cmp(&self.score),
        }
    }
}

impl PartialOrd for PostScoreIndexItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match other.publisher_canister_id.cmp(&self.publisher_canister_id) {
            Ordering::Equal => match other.post_id.cmp(&self.post_id) {
                Ordering::Equal => Some(Ordering::Equal),
                _ => Some(other.score.cmp(&self.score)),
            },
            _ => Some(other.score.cmp(&self.score)),
        }
    }
}

impl PartialEq for PostScoreIndexItem {
    fn eq(&self, other: &Self) -> bool {
        self.publisher_canister_id == other.publisher_canister_id && self.post_id == other.post_id
    }
}

impl Eq for PostScoreIndexItem {}

#[cfg(test)]
pub(crate) mod test {
    use candid::Principal;

    use super::PostScoreIndexItem;
    use std::collections::BTreeSet;

    #[test]
    fn post_score_index_items_with_different_score_treated_as_the_same_item() {
        // * exact same item
        assert_eq!(
            PostScoreIndexItem {
                score: 1,
                post_id: 1,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
            },
            PostScoreIndexItem {
                score: 1,
                post_id: 1,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
            }
        );

        // * same item with different scores
        assert_eq!(
            PostScoreIndexItem {
                score: 1,
                post_id: 1,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
            },
            PostScoreIndexItem {
                score: 2,
                post_id: 1,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
            }
        );

        // * different post_id with same score
        assert_ne!(
            PostScoreIndexItem {
                score: 1,
                post_id: 1,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
            },
            PostScoreIndexItem {
                score: 1,
                post_id: 2,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
            }
        );
    }

    #[test]
    fn post_score_index_items_when_updating_same_item_with_different_score_no_duplicates_created() {
        let mut set = BTreeSet::new();
        set.replace(PostScoreIndexItem {
            score: 18_446_744_073_709_493_716,
            post_id: 36,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });
        set.replace(PostScoreIndexItem {
            score: 18_446_744_073_704_278_166,
            post_id: 36,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });
        set.replace(PostScoreIndexItem {
            score: 18_446_744_073_605_493_716,
            post_id: 36,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });

        println!("{:?}", set);

        assert_eq!(set.len(), 1);

        set.replace(PostScoreIndexItem {
            score: 18_446_744_073_709_493_716,
            post_id: 36,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });
        set.replace(PostScoreIndexItem {
            score: 18_446_744_073_704_278_166,
            post_id: 36,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });
        set.replace(PostScoreIndexItem {
            score: 18_446_744_073_605_493_716,
            post_id: 36,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });

        assert_eq!(set.len(), 1);

        set.replace(PostScoreIndexItem {
            score: 18_446_744_073_704_278_166,
            post_id: 31,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });
        set.replace(PostScoreIndexItem {
            score: 18_446,
            post_id: 31,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });

        let second_item = set.get(&PostScoreIndexItem {
            score: 18_446,
            post_id: 31,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });

        assert_eq!(set.len(), 2);
        assert!(second_item.is_some());
        assert_eq!(second_item.unwrap().score, 18_446);
    }

    #[test]
    fn post_score_index_item_when_adding_3_items_with_duplicates() {
        let mut set = BTreeSet::new();
        set.replace(PostScoreIndexItem {
            score: 1,
            post_id: 1,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });
        set.replace(PostScoreIndexItem {
            score: 2,
            post_id: 2,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });
        set.replace(PostScoreIndexItem {
            score: 3,
            post_id: 3,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });

        assert_eq!(set.len(), 3);

        set.replace(PostScoreIndexItem {
            score: 4,
            post_id: 1,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });
        set.replace(PostScoreIndexItem {
            score: 5,
            post_id: 2,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });
        set.replace(PostScoreIndexItem {
            score: 6,
            post_id: 3,
            publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
        });

        // assert_eq!(set.len(), 3);
    }
}
