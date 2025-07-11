
use candid::Principal;
use pocket_ic::PocketIc;
use shared_utils::
    common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType}
;


pub fn get_initialized_env_with_provisioned_known_canisters(
    pocket_ic: &PocketIc, mut known_principals: KnownPrincipalMap,
) -> KnownPrincipalMap {

    let platform_canister_id = known_principals
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let global_admin = known_principals
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let user_index = pocket_ic
        .update_call(
            platform_canister_id,
            global_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[0]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                pocket_ic::WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    known_principals.insert(KnownPrincipalType::CanisterIdUserIndex, user_index);

    for _ in 0..50 {
        pocket_ic.tick();
    }

    known_principals
}