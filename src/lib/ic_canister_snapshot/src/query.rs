/// Query helpers for the IC canister snapshot database.
///
/// Call these from any test after `populate_db` has been run at least once.
use rusqlite::{params, Connection};

use crate::fetch::{db_path, open_and_init_db};

// ─── open helper ──────────────────────────────────────────────────────────────

/// Open the snapshot database.  Pass `Some(path)` to override the default path
/// or the `IC_CANISTER_DB_PATH` env var.
pub fn open_db(path: Option<&str>) -> Connection {
    let resolved = path.map(|s| s.to_string()).unwrap_or_else(db_path);
    open_and_init_db(&resolved)
}

// ─── query functions ──────────────────────────────────────────────────────────

/// Return the canister IDs of every canister whose controller list contains
/// `principal_id` (text form, e.g. `"rrkah-fqaaa-aaaaa-aaaaq-cai"`).
pub fn find_canisters_by_controller(conn: &Connection, principal_id: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare(
            "SELECT canister_id FROM controllers
             WHERE controller = ?1
             ORDER BY canister_id",
        )
        .expect("failed to prepare find_canisters_by_controller statement");

    stmt.query_map(params![principal_id], |row| row.get(0))
        .expect("query failed")
        .filter_map(|r| r.ok())
        .collect()
}

/// Return all controllers for the given canister ID.
pub fn get_controllers_for_canister(conn: &Connection, canister_id: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare(
            "SELECT controller FROM controllers
             WHERE canister_id = ?1
             ORDER BY controller",
        )
        .expect("failed to prepare get_controllers_for_canister statement");

    stmt.query_map(params![canister_id], |row| row.get(0))
        .expect("query failed")
        .filter_map(|r| r.ok())
        .collect()
}

// ─── metadata struct ──────────────────────────────────────────────────────────

/// Metadata for a single canister row.
#[derive(Debug)]
pub struct CanisterInfo {
    pub canister_id: String,
    pub api_id: Option<i64>,
    pub subnet_id: Option<String>,
    pub module_hash: Option<String>,
    pub language: Option<String>,
    pub updated_at: Option<String>,
}

/// Return metadata for a canister, or `None` if it is not in the database.
pub fn get_canister_info(conn: &Connection, canister_id: &str) -> Option<CanisterInfo> {
    conn.query_row(
        "SELECT canister_id, api_id, subnet_id, module_hash, language, updated_at
         FROM canisters WHERE canister_id = ?1",
        params![canister_id],
        |row| {
            Ok(CanisterInfo {
                canister_id: row.get(0)?,
                api_id: row.get(1)?,
                subnet_id: row.get(2)?,
                module_hash: row.get(3)?,
                language: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )
    .ok()
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Demonstrate finding canisters by controller.
    ///
    /// Uses the NNS governance canister as a well-known example controller.
    ///
    /// Run with:
    ///   cargo test -p ic_canister_snapshot test_find_canisters_by_controller -- --ignored
    #[test]
    #[ignore]
    fn test_find_canisters_by_controller() {
        let conn = open_db(None);

        // NNS governance canister controls many NNS-related canisters.
        let principal = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        let canisters = find_canisters_by_controller(&conn, principal);

        println!(
            "[test] {} controls {} canister(s)",
            principal,
            canisters.len()
        );
        for id in &canisters {
            let controllers = get_controllers_for_canister(&conn, id);
            println!("  {} => controllers: {:?}", id, controllers);
        }

        assert!(
            !canisters.is_empty(),
            "expected at least one canister controlled by {}",
            principal
        );
    }

    /// Show all controllers of a specific canister.
    ///
    /// Uses the NNS ICP ledger canister as an example.
    ///
    /// Run with:
    ///   cargo test -p ic_canister_snapshot test_get_controllers_for_canister -- --ignored
    #[test]
    #[ignore]
    fn test_get_controllers_for_canister() {
        let conn = open_db(None);

        let canister_id = "ryjl3-tyaaa-aaaaa-aaaba-cai"; // NNS ICP ledger
        let controllers = get_controllers_for_canister(&conn, canister_id);
        let info = get_canister_info(&conn, canister_id);

        println!("[test] Controllers of {}: {:?}", canister_id, controllers);
        println!("[test] Canister info: {:#?}", info);

        assert!(
            !controllers.is_empty(),
            "expected controllers for {}",
            canister_id
        );
    }
}
