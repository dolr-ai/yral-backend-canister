
use ic_cdk_macros::query;
use crate::CANISTER_DATA;

#[query]
fn get_total_number_of_posts() -> u64 {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.get_total_number_of_posts()
    })
}
mod test {
    use std::collections::HashSet;
    use shared_utils::{canister_specific::individual_user_template::types::post::{Post, PostViewStatistics}, common::types::top_posts::post_score_index_item::PostStatus};
    use std::time::SystemTime;
    use super::*;
    use crate::CanisterData;
    use candid::Principal;


    #[test]
    fn test_get_posts_of_this_user_profile_with_pagination_cursor_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.profile.principal_id = Some(Principal::anonymous());

        let posts = vec![
            Post {
                id: 1,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                is_nsfw: false,
            },
            Post {
                id: 2,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                is_nsfw: false,
            },
            Post {
                id: 3,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                is_nsfw: false,
            },
            Post {
                id: 4,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                is_nsfw: false,
            },
            Post {
                id: 5,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::BannedDueToUserReporting,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                is_nsfw: false,
            },
            Post {
                id: 6,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                is_nsfw: false,
            },
            Post {
                id: 7,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::Deleted,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                is_nsfw: false,
            },
        ];

        posts.into_iter().for_each(|post| {
            let _ = canister_data.add_post(post);
        });

        // Test with NSFW filter
        let result = canister_data.get_total_number_of_posts();
        
        assert_eq!(result, 6);
        let res = canister_data.delete_post(1);
        let result = canister_data.get_total_number_of_posts();
        
        if let Ok(()) = res {
            assert_eq!(result, 5);
        } else {
            assert_eq!(result, 6);
        }
    }
}
