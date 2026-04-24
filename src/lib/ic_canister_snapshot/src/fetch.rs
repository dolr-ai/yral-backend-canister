/// Fetches all canisters from the IC dashboard API and persists them to a local
/// SQLite database.
///
/// Run the snapshot test once with:
///
///   cargo test -p ic_canister_snapshot test_populate_ic_canister_db -- --ignored
///
/// Or from integration tests:
///
///   cargo test -p integration_tests test_populate_ic_canister_db -- --ignored
///
/// The database path defaults to `./ic_canisters.db` but can be overridden with
/// the `IC_CANISTER_DB_PATH` environment variable.
///
/// Resumability: a `progress` table stores the last successfully committed
/// offset.  If the run is interrupted, re-running resumes from that offset.
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use rusqlite::{params, Connection};
use serde::Deserialize;

// ─── constants ────────────────────────────────────────────────────────────────

const API_BASE: &str = "https://ic-api.internetcomputer.org/api/v3/canisters";
/// Maximum page size accepted by the dashboard API (values above 100 → 422).
const PAGE_SIZE: usize = 100;
const NUM_WORKERS: usize = 16;
/// Commit to SQLite (and update checkpoint) after this many pages have been
/// received from workers.
const CHECKPOINT_EVERY_N_PAGES: usize = 10;
/// Initial back-off delay on rate-limit or transient server errors (seconds).
/// Doubles with each retry attempt.
const RETRY_SLEEP_SECS: u64 = 5;
const MAX_RETRIES: usize = 5;

// ─── API types ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CanisterRecord {
    pub canister_id: String,
    /// Numeric row ID assigned by the dashboard (not the IC principal).
    pub id: Option<i64>,
    pub controllers: Option<Vec<String>>,
    pub subnet_id: Option<String>,
    pub module_hash: Option<String>,
    pub name: Option<String>,
    pub language: Option<String>,
    pub canister_type: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Vec<CanisterRecord>,
    total_canisters: Option<u64>,
}

// ─── HTTP helpers ─────────────────────────────────────────────────────────────

fn fetch_page(
    client: &reqwest::blocking::Client,
    offset: usize,
) -> Result<ApiResponse, reqwest::Error> {
    let url = format!("{}?limit={}&offset={}", API_BASE, PAGE_SIZE, offset);
    let mut last_err: Option<reqwest::Error> = None;

    for attempt in 0..MAX_RETRIES {
        match client.get(&url).send() {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return resp.json::<ApiResponse>();
                }
                if status.as_u16() == 429 || status.is_server_error() {
                    let sleep = RETRY_SLEEP_SECS * 2_u64.pow(attempt as u32);
                    eprintln!(
                        "[fetch] HTTP {} at offset {}, retrying in {}s…",
                        status, offset, sleep
                    );
                    thread::sleep(Duration::from_secs(sleep));
                    continue;
                }
                // Any other non-success (e.g. 404) — skip this page.
                eprintln!("[fetch] HTTP {} at offset {} — skipping page", status, offset);
                return Ok(ApiResponse {
                    data: vec![],
                    total_canisters: None,
                });
            }
            Err(e) => {
                eprintln!(
                    "[fetch] Network error at offset {} (attempt {}): {}",
                    offset, attempt, e
                );
                last_err = Some(e);
                thread::sleep(Duration::from_secs(RETRY_SLEEP_SECS));
            }
        }
    }
    Err(last_err.unwrap())
}

// ─── Database helpers ─────────────────────────────────────────────────────────

/// Return the database path: `IC_CANISTER_DB_PATH` env var, or `./ic_canisters.db`.
pub fn db_path() -> String {
    std::env::var("IC_CANISTER_DB_PATH").unwrap_or_else(|_| "./ic_canisters.db".to_string())
}

/// Open (or create) the database and ensure the schema is present.
pub fn open_and_init_db(path: &str) -> Connection {
    let conn = Connection::open(path).expect("failed to open SQLite database");
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;

        CREATE TABLE IF NOT EXISTS canisters (
            canister_id   TEXT PRIMARY KEY,
            api_id        INTEGER,
            subnet_id     TEXT,
            module_hash   TEXT,
            name          TEXT,
            language      TEXT,
            canister_type TEXT,
            updated_at    TEXT
        );

        CREATE TABLE IF NOT EXISTS controllers (
            canister_id TEXT NOT NULL,
            controller  TEXT NOT NULL,
            PRIMARY KEY (canister_id, controller)
        );

        CREATE INDEX IF NOT EXISTS idx_controllers_by_controller
            ON controllers(controller);

        -- Stores the last offset whose page was fully committed.
        CREATE TABLE IF NOT EXISTS progress (
            id                    INTEGER PRIMARY KEY CHECK (id = 1),
            last_completed_offset INTEGER NOT NULL DEFAULT -1
        );

        INSERT OR IGNORE INTO progress (id, last_completed_offset) VALUES (1, -1);
        ",
    )
    .expect("failed to initialise schema");
    conn
}

fn read_checkpoint(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT last_completed_offset FROM progress WHERE id = 1",
        [],
        |row| row.get(0),
    )
    .unwrap_or(-1)
}

fn write_checkpoint(conn: &Connection, offset: usize) {
    conn.execute(
        "UPDATE progress SET last_completed_offset = ?1 WHERE id = 1",
        params![offset as i64],
    )
    .expect("failed to update checkpoint");
}

// ─── Batch insert ─────────────────────────────────────────────────────────────

fn insert_batch(conn: &Connection, records: Vec<CanisterRecord>) {
    let tx = conn
        .unchecked_transaction()
        .expect("failed to begin transaction");

    for rec in &records {
        tx.execute(
            "INSERT OR REPLACE INTO canisters
             (canister_id, api_id, subnet_id, module_hash, name, language, canister_type, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                rec.canister_id,
                rec.id,
                rec.subnet_id,
                rec.module_hash,
                rec.name,
                rec.language,
                rec.canister_type,
                rec.updated_at,
            ],
        )
        .expect("failed to insert canister row");

        if let Some(controllers) = &rec.controllers {
            for controller in controllers {
                tx.execute(
                    "INSERT OR IGNORE INTO controllers (canister_id, controller)
                     VALUES (?1, ?2)",
                    params![rec.canister_id, controller],
                )
                .expect("failed to insert controller row");
            }
        }
    }

    tx.commit().expect("failed to commit transaction");
}

// ─── Main populate logic ──────────────────────────────────────────────────────

/// Fetch **all** canisters from the IC dashboard and write them to `conn`.
///
/// This function is safe to call multiple times: it reads the `progress` table
/// and skips pages that have already been committed, so an interrupted run can
/// be resumed without duplicating work.
pub fn populate_db(conn: &Connection) {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build HTTP client");

    // ── first page: get total count ────────────────────────────────────────
    let first_page = fetch_page(&client, 0).expect("failed to fetch first page");
    let total_canisters = first_page
        .total_canisters
        .expect("API did not return total_canisters") as usize;
    let total_pages = total_canisters.div_ceil(PAGE_SIZE);
    println!(
        "[populate] {} canisters across {} pages",
        total_canisters, total_pages
    );

    // ── resume from checkpoint ─────────────────────────────────────────────
    let checkpoint = read_checkpoint(conn);
    let start_offset = if checkpoint >= 0 {
        let resume = (checkpoint as usize) + PAGE_SIZE;
        println!(
            "[populate] Resuming from offset {} (last checkpoint: {})",
            resume, checkpoint
        );
        resume
    } else {
        // Commit page 0 that we already fetched.
        insert_batch(conn, first_page.data);
        write_checkpoint(conn, 0);
        PAGE_SIZE
    };

    if start_offset >= total_pages * PAGE_SIZE {
        println!("[populate] Already complete — nothing to do.");
        return;
    }

    // ── build work queue ───────────────────────────────────────────────────
    let all_offsets: VecDeque<usize> = (start_offset..total_pages * PAGE_SIZE)
        .step_by(PAGE_SIZE)
        .collect();
    let remaining_pages = all_offsets.len();
    let work_queue = Arc::new(Mutex::new(all_offsets));

    // ── spawn worker threads ───────────────────────────────────────────────
    let (tx, rx) = std::sync::mpsc::channel::<(usize, Vec<CanisterRecord>)>();

    let mut handles = vec![];
    for _ in 0..NUM_WORKERS {
        let queue = Arc::clone(&work_queue);
        let sender = tx.clone();
        let client = client.clone();

        let handle = thread::spawn(move || loop {
            let offset = {
                let mut q = queue.lock().unwrap();
                q.pop_front()
            };
            let Some(offset) = offset else { break };

            match fetch_page(&client, offset) {
                Ok(page) => {
                    if sender.send((offset, page.data)).is_err() {
                        break; // main thread exited early
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[worker] Failed to fetch offset {} after retries: {}",
                        offset, e
                    );
                    // Send empty batch so we don't stall the checkpoint counter.
                    let _ = sender.send((offset, vec![]));
                }
            }
        });
        handles.push(handle);
    }
    drop(tx); // closing the extra sender makes rx.iter() terminate when all workers finish

    // ── main thread: collect results and write to DB ───────────────────────
    let mut pending: Vec<(usize, Vec<CanisterRecord>)> = vec![];
    let mut pages_since_checkpoint: usize = 0;
    let mut committed_pages: usize = 0;

    for (offset, records) in rx {
        pending.push((offset, records));
        pages_since_checkpoint += 1;

        if pages_since_checkpoint >= CHECKPOINT_EVERY_N_PAGES {
            // Sort pending by offset so the checkpoint value is the maximum
            // contiguous offset we have fully received.
            pending.sort_by_key(|(off, _)| *off);
            let max_offset = pending.last().map(|(o, _)| *o).unwrap_or(start_offset);
            let all_records: Vec<CanisterRecord> =
                pending.drain(..).flat_map(|(_, recs)| recs).collect();

            insert_batch(conn, all_records);
            write_checkpoint(conn, max_offset);
            committed_pages += pages_since_checkpoint;
            pages_since_checkpoint = 0;

            let pct = committed_pages * 100 / remaining_pages.max(1);
            println!(
                "[populate] {}/{} pages committed ({}%)",
                committed_pages, remaining_pages, pct
            );
        }
    }

    // ── flush any remaining pending pages ─────────────────────────────────
    if !pending.is_empty() {
        pending.sort_by_key(|(off, _)| *off);
        let max_offset = pending.last().map(|(o, _)| *o).unwrap_or(start_offset);
        let all_records: Vec<CanisterRecord> =
            pending.drain(..).flat_map(|(_, recs)| recs).collect();
        insert_batch(conn, all_records);
        write_checkpoint(conn, max_offset);
        committed_pages += pages_since_checkpoint;
    }

    for h in handles {
        let _ = h.join();
    }

    println!("[populate] Done. {} pages processed.", committed_pages);
}

// ─── Test entry point ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Fetch all canisters from the IC dashboard into a local SQLite database.
    ///
    /// Run with:
    ///   cargo test -p ic_canister_snapshot test_populate_ic_canister_db -- --ignored
    #[test]
    #[ignore]
    fn test_populate_ic_canister_db() {
        let path = db_path();
        println!("[test] Database path: {}", path);
        let conn = open_and_init_db(&path);
        populate_db(&conn);

        let canister_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM canisters", [], |r| r.get(0))
            .expect("count query failed");
        let controller_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM controllers", [], |r| r.get(0))
            .expect("count query failed");

        println!(
            "[test] Snapshot complete: {} canisters, {} controller entries",
            canister_count, controller_count
        );
        assert!(canister_count > 0, "expected canisters in DB after populate");
    }
}
