#[cfg(test)]
mod test {

    use std::{
        collections::{BTreeMap, BTreeSet, HashMap, HashSet},
        time::SystemTime,
    };

    use candid::{Nat, Principal};
    use ic_cdk::api::management_canister::main::CanisterId;
    use shared_utils::{
        canister_specific::{
            individual_user_template::types::{
                cdao::{AirdropInfo, ClaimStatus, DeployedCdaoCanisters},
                cents::CentsToken,
                follow::FollowEntryDetail,
                hot_or_not::{
                    BetDetails, BetDirection, BetOutcomeForBetMaker, BetPayout, GlobalBetId,
                    GlobalRoomId, PlacedBetDetail, RoomBetPossibleOutcomes, RoomDetailsV1,
                    SlotDetailsV1, SlotId, StablePrincipal,
                },
                migration::MigrationInfo,
                post::PostViewStatistics,
                profile::{UserProfile, UserProfileGlobalStats},
                pump_n_dump::{GameDirection, ParticipatedGameInfo},
                session::SessionType,
            },
            user_info_service::types::{NSFWInfo, ProfilePictureData, SubscriptionPlan},
        },
        common::types::{
            app_primitive_type::PostId,
            known_principal::KnownPrincipalType,
            top_posts::post_score_index_item::{PostScoreIndexItem, PostStatus},
            utility_token::token_event::{MintEvent, TokenEvent},
            version_details::VersionDetails,
        },
    };
    use test_utils::setup::test_constants::get_mock_user_alice_canister_id;

    use crate::{
        api::snapshot::{CanisterDataForSnapshot, PostForSnapshot, TokenBetGameForSnapshot},
        data_model::{
            pump_n_dump::{NatStore, TokenBetGame},
            CanisterData,
        },
    };

    #[test]
    fn test_serde_json_snapshot_canister_data() {
        let mut created_posts = BTreeMap::<u64, PostForSnapshot>::new();

        let temp_principal = get_mock_user_alice_canister_id();

        let post1 = PostForSnapshot {
            id: 1,
            description: "todo".to_string(),
            hashtags: vec!["dfasf".to_string()],
            video_uid: "21".to_string(),
            status: PostStatus::ReadyToView,
            created_at: SystemTime::now(),
            likes: HashSet::from([temp_principal]),
            share_count: 12,
            view_stats: PostViewStatistics {
                total_view_count: 12,
                threshold_view_count: 12,
                average_watch_percentage: 21,
            },
            is_nsfw: false,
        };
        created_posts.insert(1, post1);

        let mut room_details_map: BTreeMap<GlobalRoomId, RoomDetailsV1> = BTreeMap::new();
        let global_room_id = GlobalRoomId(1, 1, 1);
        room_details_map.insert(
            global_room_id,
            RoomDetailsV1 {
                bet_outcome: RoomBetPossibleOutcomes::HotWon,
                room_bets_total_pot: 12000,
                total_hot_bets: 31,
                total_not_bets: 30,
            },
        );

        let mut bet_details_map: BTreeMap<GlobalBetId, BetDetails> = BTreeMap::new();
        let global_bet_id = GlobalBetId(global_room_id, StablePrincipal(temp_principal));
        bet_details_map.insert(
            global_bet_id,
            BetDetails {
                amount: 100,
                bet_direction: BetDirection::Hot,
                payout: BetPayout::Calculated(1000),
                bet_maker_canister_id: temp_principal,
                bet_maker_informed_status: None,
            },
        );

        let mut post_principal_map: BTreeMap<(PostId, StablePrincipal), ()> = BTreeMap::new();
        post_principal_map.insert((1, StablePrincipal(temp_principal)), ());

        let mut slot_details_map: BTreeMap<(PostId, SlotId), SlotDetailsV1> = BTreeMap::new();
        slot_details_map.insert((1, 1), SlotDetailsV1 { active_room_id: 2 });

        let mut all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail> =
            BTreeMap::new();
        all_hot_or_not_bets_placed.insert(
            (temp_principal, 1),
            PlacedBetDetail {
                canister_id: temp_principal,
                post_id: 1,
                slot_id: 1,
                room_id: 1,
                amount_bet: 100,
                bet_direction: BetDirection::Hot,
                bet_placed_at: SystemTime::now(),
                outcome_received: BetOutcomeForBetMaker::Won(10),
            },
        );

        let mut follow_sorted_index = BTreeMap::<u64, FollowEntryDetail>::new();
        follow_sorted_index.insert(
            1,
            FollowEntryDetail {
                principal_id: temp_principal,
                canister_id: temp_principal,
            },
        );
        let mut follow_members = HashMap::<FollowEntryDetail, u64>::new();
        follow_members.insert(
            FollowEntryDetail {
                principal_id: temp_principal,
                canister_id: temp_principal,
            },
            1,
        );

        let known_principal_ids = HashMap::<KnownPrincipalType, Principal>::new();

        let mut utility_history = BTreeMap::<u64, TokenEvent>::new();
        utility_history.insert(
            1,
            TokenEvent::Mint {
                amount: 10,
                details: MintEvent::NewUserSignup {
                    new_user_principal_id: temp_principal,
                },
                timestamp: SystemTime::now(),
            },
        );

        let mut items_sorted_by_score = BTreeMap::<u64, Vec<PostScoreIndexItem>>::new();
        items_sorted_by_score.insert(
            1,
            vec![PostScoreIndexItem {
                score: 100,
                post_id: 1,
                publisher_canister_id: temp_principal,
            }],
        );

        let mut item_prescence_index = HashMap::<(Principal, u64), u64>::new();
        item_prescence_index.insert((temp_principal, 1), 1);

        let mut principal_list = BTreeSet::<Principal>::new();
        principal_list.insert(temp_principal);

        let mut airdrop_info = AirdropInfo::default();
        airdrop_info
            .principals_who_successfully_claimed
            .insert(temp_principal, ClaimStatus::Claimed);

        let cdao_canisters = vec![DeployedCdaoCanisters {
            governance: temp_principal,
            ledger: temp_principal,
            root: temp_principal,
            swap: temp_principal,
            index: temp_principal,
            airdrop_info,
        }];

        let mut token_roots = BTreeMap::<Principal, ()>::new();
        token_roots.insert(temp_principal, ());

        let canister_data_snapshot = CanisterDataForSnapshot {
            all_created_posts: created_posts,
            known_principal_ids,
            profile: UserProfile {
                principal_id: Some(temp_principal),
                profile_stats: UserProfileGlobalStats {
                    hot_bets_received: 100,
                    not_bets_received: 100,
                },
                referrer_details: None,
                bio: None,
                website_url: None,
                subscription_plan: SubscriptionPlan::Free,
                profile_picture: Some(ProfilePictureData {
                    url: "dadfk".to_string(),
                    nsfw_info: NSFWInfo::default(),
                }),
                is_ai_influencer: false,
            },
            version_details: VersionDetails {
                version_number: 1,
                version: "1.0.0".to_string(),
            },
            session_type: Some(SessionType::RegisteredSession),
            last_access_time: Some(SystemTime::now()),
            migration_info: MigrationInfo::NotMigrated,
            cdao_canisters,
            token_roots,
        };

        let serde_str = serde_json::to_string(&canister_data_snapshot);
        assert!(serde_str.is_ok());

        let canister_data_snapshot: CanisterDataForSnapshot =
            serde_json::from_str(serde_str.unwrap().as_str()).unwrap();

        let canister_data = CanisterData::from(canister_data_snapshot.clone());

        // convert back to CanisterDataForSnapshot
        let canister_data_snapshot_2 = CanisterDataForSnapshot::from(&canister_data);

        // caonvert to bytes and assert_eq
        let canister_data_snapshot_bytes = serde_json::to_vec(&canister_data_snapshot).unwrap();
        let canister_data_snapshot_2_bytes = serde_json::to_vec(&canister_data_snapshot_2).unwrap();
        assert_eq!(canister_data_snapshot_bytes, canister_data_snapshot_2_bytes);
    }

    #[test]
    fn test_serde_json_snapshot_token_bet_game() {
        let mut room_details_map: BTreeMap<GlobalRoomId, RoomDetailsV1> = BTreeMap::new();
        let global_room_id = GlobalRoomId(1, 1, 1);
        room_details_map.insert(
            global_room_id,
            RoomDetailsV1 {
                bet_outcome: RoomBetPossibleOutcomes::HotWon,
                room_bets_total_pot: 15000,
                total_hot_bets: 40,
                total_not_bets: 35,
            },
        );

        let temp_principal = get_mock_user_alice_canister_id();
        let mut bet_details_map: BTreeMap<GlobalBetId, BetDetails> = BTreeMap::new();
        let global_bet_id = GlobalBetId(global_room_id, StablePrincipal(temp_principal));
        bet_details_map.insert(
            global_bet_id,
            BetDetails {
                amount: 200,
                bet_direction: BetDirection::Hot,
                payout: BetPayout::Calculated(2000),
                bet_maker_canister_id: temp_principal,
                bet_maker_informed_status: None,
            },
        );

        let mut post_principal_map: BTreeMap<(PostId, StablePrincipal), ()> = BTreeMap::new();
        post_principal_map.insert((1, StablePrincipal(temp_principal)), ());

        let mut slot_details_map: BTreeMap<(PostId, SlotId), SlotDetailsV1> = BTreeMap::new();
        slot_details_map.insert((1, 1), SlotDetailsV1 { active_room_id: 2 });

        let mut all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail> =
            BTreeMap::new();
        all_hot_or_not_bets_placed.insert(
            (temp_principal, 1),
            PlacedBetDetail {
                canister_id: temp_principal,
                post_id: 1,
                slot_id: 1,
                room_id: 1,
                amount_bet: 200,
                bet_direction: BetDirection::Hot,
                bet_placed_at: SystemTime::now(),
                outcome_received: BetOutcomeForBetMaker::Won(20),
            },
        );

        let mut liquidity_pools: BTreeMap<Principal, NatStore> = BTreeMap::new();
        liquidity_pools.insert(temp_principal, NatStore::default());

        let games = vec![ParticipatedGameInfo {
            pumps: 10,
            dumps: 25,
            reward: 1000,
            token_root: temp_principal,
            game_direction: GameDirection::Pump,
        }];

        let token_bet_game_snapshot = TokenBetGameForSnapshot {
            referral_reward: Nat::from(1000u32),
            onboarding_reward: Nat::from(500u32),
            games,
            total_dumps: Nat::from(10u32),
            total_pumps: Nat::from(25u32),
            liquidity_pools,
            cents: CentsToken::default(),
        };

        let serde_str_res = serde_json::to_string(&token_bet_game_snapshot);
        assert!(serde_str_res.is_ok(), "Serialization failed");
        let serde_str = serde_str_res.unwrap();

        let deserialized_snapshot_res: Result<TokenBetGameForSnapshot, _> =
            serde_json::from_str(&serde_str);
        assert!(
            deserialized_snapshot_res.is_ok(),
            "Deserialization failed: {:?}",
            deserialized_snapshot_res.err()
        );

        let _deserialized_snapshot = deserialized_snapshot_res.unwrap();

        let token_bet_game = TokenBetGame::from(token_bet_game_snapshot.clone());

        let token_bet_game_snapshot_2: TokenBetGameForSnapshot =
            TokenBetGameForSnapshot::from(&token_bet_game);

        // convert back to bytes and assert_eq
        let token_bet_game_snapshot_bytes = serde_json::to_vec(&token_bet_game_snapshot).unwrap();
        let token_bet_game_snapshot_2_bytes =
            serde_json::to_vec(&token_bet_game_snapshot_2).unwrap();
        assert_eq!(
            token_bet_game_snapshot_bytes,
            token_bet_game_snapshot_2_bytes
        );
    }
}
