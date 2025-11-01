use candid::{CandidType, Deserialize, Principal};
use core::fmt;
use serde::{
    de::{self, EnumAccess, VariantAccess, Visitor},
    Serialize,
};
use std::{cmp::Ordering, marker::PhantomData, time::SystemTime};

#[derive(Serialize, CandidType, Clone, Default, Debug, PartialEq, Eq, Hash, Copy)]
pub enum PostStatus {
    #[default]
    Uploaded,
    Transcoding,
    CheckingExplicitness,
    BannedForExplicitness,
    ReadyToView,
    BannedDueToUserReporting,
    Deleted,
    Published,
    Draft,
}

// This is what the #[derive(Deserialize)] macro expands to:
impl<'de> Deserialize<'de> for PostStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        // Define the possible field names for the enum variants
        const VARIANTS: &'static [&'static str] = &[
            "Uploaded",
            "Transcoding",
            "CheckingExplicitness",
            "BannedForExplicitness",
            "ReadyToView",
            "BannedDueToUserReporting",
            "Deleted",
            "Draft",
            "Published",
        ];

        enum Field {
            field0,
            field1,
            field2,
            field3,
            field4,
            field5,
            field6,
            field7,
            field8,
        }

        struct FieldVisitor;

        // Create a visitor struct to handle the deserialization

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("variant of PostStatus")
            }

            // Handle string-based deserialization (for JSON, etc.)
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "Uploaded" => Ok(Field::field0),
                    "Transcoding" => Ok(Field::field1),
                    "CheckingExplicitness" => Ok(Field::field2),
                    "BannedForExplicitness" => Ok(Field::field3),
                    "ReadyToView" => Ok(Field::field4),
                    "BannedDueToUserReporting" => Ok(Field::field5),
                    "Deleted" => Ok(Field::field6),
                    "Draft" => Ok(Field::field7),
                    "Published" => Ok(Field::field8),
                    _ => Err(de::Error::unknown_variant(value, VARIANTS)),
                }
            }

            // Handle integer-based deserialization (for binary formats)
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(Field::field0),
                    1 => Ok(Field::field1),
                    2 => Ok(Field::field2),
                    3 => Ok(Field::field3),
                    4 => Ok(Field::field4),
                    5 => Ok(Field::field5),
                    6 => Ok(Field::field6),
                    7 => Ok(Field::field7),
                    8 => Ok(Field::field8),
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 9",
                    )),
                }
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    b"Uploaded" => Ok(Field::field0),
                    b"Transcoding" => Ok(Field::field1),
                    b"CheckingExplicitness" => Ok(Field::field2),
                    b"BannedForExplicitness" => Ok(Field::field3),
                    b"ReadyToView" => Ok(Field::field4),
                    b"BannedDueToUserReporting" => Ok(Field::field5),
                    b"Deleted" => Ok(Field::field6),
                    b"Draft" => Ok(Field::field7),
                    b"Published" => Ok(Field::field8),
                    _ => Err(de::Error::unknown_variant(
                        std::str::from_utf8(v).unwrap_or("<invalid utf8>"),
                        VARIANTS,
                    )),
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct __Visitor<'de> {
            marker: PhantomData<PostStatus>,
            lifetime: PhantomData<&'de ()>,
        }
        #[automatically_derived]
        impl<'de> Visitor<'de> for __Visitor<'de> {
            type Value = PostStatus;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum PostStatus")
            }
            fn visit_enum<D>(self, data: D) -> Result<Self::Value, D::Error>
            where
                D: EnumAccess<'de>,
            {
                match EnumAccess::variant(data)? {
                    (Field::field0, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(PostStatus::Published)
                    }
                    (Field::field1, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(PostStatus::Transcoding)
                    }
                    (Field::field2, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(PostStatus::CheckingExplicitness)
                    }
                    (Field::field3, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(PostStatus::BannedForExplicitness)
                    }
                    (Field::field4, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(PostStatus::Published)
                    }
                    (Field::field5, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(PostStatus::BannedDueToUserReporting)
                    }
                    (Field::field6, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(PostStatus::Deleted)
                    }
                    (Field::field7, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(PostStatus::Draft)
                    }
                    (Field::field8, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(PostStatus::Published)
                    }
                }
            }
        }

        deserializer.deserialize_enum(
            "PostStatus",
            VARIANTS,
            __Visitor {
                marker: PhantomData::<PostStatus>,
                lifetime: PhantomData,
            },
        )

        // Use the deserializer to deserialize using our visitor
    }
}

// This is what the #[derive(Deserialize)] macro expands to:

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
    use super::PostStatus;
    use std::collections::BTreeSet;

    #[test]
    fn test_cbor_serialization_roundtrip() {
        use ciborium::{de::from_reader, ser::into_writer};

        let item = PostStatus::ReadyToView;
        let expected_decoded = PostStatus::Published;

        let mut buf = Vec::new();
        into_writer(&item, &mut buf).expect("Failed to serialize PostScoreIndexItem");

        let decoded: PostStatus =
            from_reader(buf.as_slice()).expect("Failed to deserialize PostScoreIndexItem");

        assert_eq!(decoded, expected_decoded);
    }

    #[test]
    fn add_tests_for_candid_serialization() {
        use candid::{Decode, Encode};

        let item = PostStatus::Published;

        let encoded = Encode!(&item).expect("Failed to encode PostScoreIndexItem");
        let decoded: PostStatus =
            Decode!(&encoded, PostStatus).expect("Failed to decode PostScoreIndexItem");

        assert_eq!(item, decoded);
    }

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
