pub mod memory;
use candid::Principal;
use ic_stable_structures::StableBTreeMap;
use memory::{Memory, get_property_configs_memory, get_rate_limits_memory};
use serde::{Deserialize, Serialize};
use shared_utils::canister_specific::rate_limits::types::{RateLimitEntry, RateLimitKey};
use shared_utils::canister_specific::rate_limits::{
    GlobalRateLimitConfig, PropertyRateLimitConfig, RateLimitConfig, RateLimitStatus,
};
use shared_utils::service::SetVersion;

#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    #[serde(skip, default = "init_rate_limits")]
    pub rate_limits: StableBTreeMap<RateLimitKey, RateLimitEntry, Memory>,
    #[serde(skip, default = "init_property_configs")]
    pub property_configs: StableBTreeMap<String, PropertyRateLimitConfig, Memory>,
    pub version: String,
    pub default_config: GlobalRateLimitConfig,
    pub user_info_canister: Principal,
}

impl SetVersion for CanisterData {
    fn set_version(&mut self, version: &str) {
        self.version = version.into();
    }
}

fn init_rate_limits() -> StableBTreeMap<RateLimitKey, RateLimitEntry, Memory> {
    StableBTreeMap::init(get_rate_limits_memory())
}

fn init_property_configs() -> StableBTreeMap<String, PropertyRateLimitConfig, Memory> {
    StableBTreeMap::init(get_property_configs_memory())
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            rate_limits: init_rate_limits(),
            property_configs: init_property_configs(),
            version: "v1.0.0".into(),
            default_config: GlobalRateLimitConfig::default(),
            user_info_canister: Principal::anonymous(), // Will be set during init
        }
    }
}

impl CanisterData {
    pub fn is_rate_limited_with_property(
        &self,
        principal: &Principal,
        property: &str,
        is_registered: bool,
    ) -> bool {
        if let Some(status) =
            self.get_rate_limit_status_with_property(principal, property, is_registered)
        {
            status.is_limited
        } else {
            false
        }
    }

    pub fn increment_request_with_property(&mut self, principal: &Principal, property: &str) {
        let current_time = ic_cdk::api::time() / 1_000_000_000; // Convert nanoseconds to seconds
        let key = RateLimitKey::new(*principal, property.to_string());

        let entry = if let Some(mut existing) = self.rate_limits.get(&key) {
            // Determine window duration from config, property config, or default
            let window_duration = if let Some(config) = &existing.config {
                config.window_duration_seconds
            } else if let Some(prop_config) = self.property_configs.get(&property.to_string()) {
                prop_config.window_duration_seconds
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

        self.rate_limits.insert(key, entry);
    }

    pub fn get_rate_limit_status_with_property(
        &self,
        principal: &Principal,
        property: &str,
        is_registered: bool,
    ) -> Option<RateLimitStatus> {
        let key = RateLimitKey::new(*principal, property.to_string());
        self.rate_limits.get(&key).map(|entry| {
            // Get max requests from custom config, property config, or default based on registration status
            let max_requests = if let Some(config) = &entry.config {
                config.max_requests_per_window
            } else if let Some(prop_config) = self.property_configs.get(&property.to_string()) {
                // Use property-specific config based on registration status
                if is_registered {
                    prop_config.max_requests_per_window_registered
                } else {
                    prop_config.max_requests_per_window_unregistered
                }
            } else {
                // Use default config based on registration status
                if is_registered {
                    self.default_config.max_requests_per_window_registered
                } else {
                    self.default_config.max_requests_per_window_unregistered
                }
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

    pub fn reset_rate_limit_with_property(&mut self, principal: &Principal, property: &str) {
        let key = RateLimitKey::new(*principal, property.to_string());
        self.rate_limits.remove(&key);
    }

    pub fn reset_all_principal_rate_limits(&mut self, principal: &Principal) {
        // Remove all entries for a principal across all properties
        let keys_to_remove: Vec<RateLimitKey> = self
            .rate_limits
            .iter()
            .filter(|(k, _)| k.principal == *principal)
            .map(|(k, _)| k)
            .collect();

        for key in keys_to_remove {
            self.rate_limits.remove(&key);
        }
    }

    pub fn clear_all_rate_limits(&mut self) {
        // Clear all entries from the StableBTreeMap
        let keys: Vec<RateLimitKey> = self.rate_limits.iter().map(|(k, _)| k).collect();
        for key in keys {
            self.rate_limits.remove(&key);
        }
    }

    pub fn set_principal_property_rate_limit(
        &mut self,
        principal: &Principal,
        property: &str,
        config: RateLimitConfig,
    ) {
        let key = RateLimitKey::new(*principal, property.to_string());
        if let Some(mut entry) = self.rate_limits.get(&key) {
            entry.config = Some(config);
            self.rate_limits.insert(key, entry);
        } else {
            let current_time = ic_cdk::api::time() / 1_000_000_000;
            let entry = RateLimitEntry {
                request_count: 0,
                window_start: current_time,
                config: Some(config),
            };
            self.rate_limits.insert(key, entry);
        }
    }

    pub fn get_principal_property_config(
        &self,
        principal: &Principal,
        property: &str,
    ) -> Option<RateLimitConfig> {
        let key = RateLimitKey::new(*principal, property.to_string());
        self.rate_limits
            .get(&key)
            .and_then(|entry| entry.config.clone())
    }

    pub fn set_property_config(&mut self, config: PropertyRateLimitConfig) {
        self.property_configs
            .insert(config.property.clone(), config);
    }

    pub fn get_property_config(&self, property: &str) -> Option<PropertyRateLimitConfig> {
        self.property_configs
            .get(&property.to_string())
            .map(|c| c.clone())
    }

    pub fn remove_property_config(&mut self, property: &str) {
        self.property_configs.remove(&property.to_string());
    }

    pub fn get_all_property_configs(&self) -> Vec<PropertyRateLimitConfig> {
        self.property_configs
            .iter()
            .map(|(_, v)| v.clone())
            .collect()
    }
}
