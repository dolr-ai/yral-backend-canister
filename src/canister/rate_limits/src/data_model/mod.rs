pub mod memory;
use crate::RateLimitStatus;
use candid::{CandidType, Principal};
use ic_stable_structures::Storable;
use ic_stable_structures::{StableBTreeMap, storable::Bound};
use memory::{Memory, get_rate_limits_memory};
use serde::{Deserialize, Serialize};
use shared_utils::service::SetVersion;
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Clone, Debug, CandidType)]
pub struct GlobalRateLimitConfig {
    pub max_requests_per_window_registered: u64,
    pub max_requests_per_window_unregistered: u64,
    pub window_duration_seconds: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, CandidType)]
pub struct RateLimitConfig {
    pub max_requests_per_window: u64,
    pub window_duration_seconds: u64,
}

impl Default for GlobalRateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_window_registered: 5,   // Default: 5 requests
            max_requests_per_window_unregistered: 1, // Default: 1 request
            window_duration_seconds: 86400,          // Default: 1 day window
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RateLimitEntry {
    pub request_count: u64,
    pub window_start: u64,
    pub config: Option<RateLimitConfig>,
}

impl Storable for RateLimitEntry {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = serde_json::to_vec(self).expect("Failed to serialize RateLimitEntry");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize RateLimitEntry")
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    #[serde(skip, default = "init_rate_limits")]
    pub rate_limits: StableBTreeMap<Principal, RateLimitEntry, Memory>,
    pub version: String,
    pub default_config: GlobalRateLimitConfig,
}

impl SetVersion for CanisterData {
    fn set_version(&mut self, version: &str) {
        self.version = version.into();
    }
}

fn init_rate_limits() -> StableBTreeMap<Principal, RateLimitEntry, Memory> {
    StableBTreeMap::init(get_rate_limits_memory())
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            rate_limits: init_rate_limits(),
            version: "v1.0.0".into(),
            default_config: GlobalRateLimitConfig::default(),
        }
    }
}

impl CanisterData {
    pub fn is_rate_limited(&self, principal: &Principal, is_registered: bool) -> bool {
        let current_time = ic_cdk::api::time() / 1_000_000_000; // Convert nanoseconds to seconds

        if let Some(entry) = self.rate_limits.get(principal) {
            // Use custom config if available, otherwise use default
            let (max_requests, window_duration) = if let Some(config) = &entry.config {
                (config.max_requests_per_window, config.window_duration_seconds)
            } else {
                // Use default config based on registration status
                if is_registered {
                    (self.default_config.max_requests_per_window_registered, self.default_config.window_duration_seconds)
                } else {
                    (self.default_config.max_requests_per_window_unregistered, self.default_config.window_duration_seconds)
                }
            };

            // Check if we're still in the same window
            if current_time < entry.window_start + window_duration {
                return entry.request_count >= max_requests;
            }
        }
        false
    }

    pub fn increment_request(&mut self, principal: &Principal) {
        let current_time = ic_cdk::api::time() / 1_000_000_000; // Convert nanoseconds to seconds

        let entry = if let Some(mut existing) = self.rate_limits.get(principal) {
            // Determine window duration from config or default
            let window_duration = if let Some(config) = &existing.config {
                config.window_duration_seconds
            } else {
                self.default_config.window_duration_seconds
            };

            // Check if we need to reset the window
            if current_time >= existing.window_start + window_duration {
                RateLimitEntry {
                    request_count: 1,
                    window_start: current_time,
                    config: existing.config.clone(),
                }
            } else {
                existing.request_count += 1;
                existing
            }
        } else {
            RateLimitEntry {
                request_count: 1,
                window_start: current_time,
                config: None, // Use default config for new entries
            }
        };

        self.rate_limits.insert(*principal, entry);
    }

    pub fn get_rate_limit_status(&self, principal: &Principal) -> Option<RateLimitStatus> {
        self.rate_limits.get(principal).map(|entry| {
            // Get max requests from custom config or default
            let max_requests = if let Some(config) = &entry.config {
                config.max_requests_per_window
            } else {
                // For status reporting, we'll use the registered limit as a default
                self.default_config.max_requests_per_window_registered
            };
            
            let is_limited = entry.request_count >= max_requests;
            RateLimitStatus {
                principal: *principal,
                request_count: entry.request_count,
                window_start: entry.window_start,
                is_limited,
            }
        })
    }

    pub fn reset_rate_limit(&mut self, principal: &Principal) {
        self.rate_limits.remove(principal);
    }

    pub fn clear_all_rate_limits(&mut self) {
        // Clear all entries from the StableBTreeMap
        let principals: Vec<Principal> = self.rate_limits.iter().map(|(k, _)| k).collect();
        for principal in principals {
            self.rate_limits.remove(&principal);
        }
    }

    pub fn set_principal_rate_limit(&mut self, principal: &Principal, config: RateLimitConfig) {
        if let Some(mut entry) = self.rate_limits.get(principal) {
            entry.config = Some(config);
            self.rate_limits.insert(*principal, entry);
        } else {
            let current_time = ic_cdk::api::time() / 1_000_000_000;
            let entry = RateLimitEntry {
                request_count: 0,
                window_start: current_time,
                config: Some(config),
            };
            self.rate_limits.insert(*principal, entry);
        }
    }

    pub fn get_principal_config(&self, principal: &Principal) -> Option<RateLimitConfig> {
        self.rate_limits
            .get(principal)
            .and_then(|entry| entry.config.clone())
    }
}
