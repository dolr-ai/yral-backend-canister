pub mod memory;
use candid::Principal;
use ic_stable_structures::Storable;
use ic_stable_structures::{StableBTreeMap, storable::Bound};
use memory::{Memory, get_rate_limits_memory, get_property_configs_memory};
use serde::{Deserialize, Serialize};
use shared_utils::canister_specific::rate_limits::{
    GlobalRateLimitConfig as SharedGlobalRateLimitConfig, 
    PropertyRateLimitConfig as SharedPropertyRateLimitConfig, 
    RateLimitConfig, RateLimitStatus,
};
use shared_utils::service::SetVersion;
use std::borrow::Cow;

// Wrapper types to implement local traits
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalRateLimitConfig(pub SharedGlobalRateLimitConfig);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PropertyRateLimitConfig(pub SharedPropertyRateLimitConfig);

impl Default for GlobalRateLimitConfig {
    fn default() -> Self {
        GlobalRateLimitConfig(SharedGlobalRateLimitConfig {
            max_requests_per_window_registered: 5,
            max_requests_per_window_unregistered: 1,
            window_duration_seconds: 86400,
        })
    }
}

impl std::ops::Deref for GlobalRateLimitConfig {
    type Target = SharedGlobalRateLimitConfig;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for GlobalRateLimitConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for PropertyRateLimitConfig {
    type Target = SharedPropertyRateLimitConfig;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PropertyRateLimitConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitKey {
    pub principal: Principal,
    pub property: String,
}

impl RateLimitKey {
    pub fn new(principal: Principal, property: String) -> Self {
        Self { principal, property }
    }
    
    pub fn default_property(principal: Principal) -> Self {
        Self {
            principal,
            property: "default".to_string(),
        }
    }
}

impl Storable for RateLimitKey {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = serde_json::to_vec(self).expect("Failed to serialize RateLimitKey");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize RateLimitKey")
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for PropertyRateLimitConfig {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = serde_json::to_vec(&self.0).expect("Failed to serialize PropertyRateLimitConfig");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let inner: SharedPropertyRateLimitConfig = serde_json::from_slice(&bytes).expect("Failed to deserialize PropertyRateLimitConfig");
        PropertyRateLimitConfig(inner)
    }

    const BOUND: Bound = Bound::Unbounded;
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
    pub fn is_rate_limited_with_property(&self, principal: &Principal, property: &str, is_registered: bool) -> bool {
        let current_time = ic_cdk::api::time() / 1_000_000_000; // Convert nanoseconds to seconds
        let key = RateLimitKey::new(*principal, property.to_string());

        if let Some(entry) = self.rate_limits.get(&key) {
            // Use custom config if available, otherwise check property config, then default
            let (max_requests, window_duration) = if let Some(config) = &entry.config {
                (config.max_requests_per_window, config.window_duration_seconds)
            } else if let Some(prop_config) = self.property_configs.get(&property.to_string()) {
                // Use property-specific config based on registration status
                if is_registered {
                    (prop_config.max_requests_per_window_registered, prop_config.window_duration_seconds)
                } else {
                    (prop_config.max_requests_per_window_unregistered, prop_config.window_duration_seconds)
                }
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

    pub fn get_rate_limit_status_with_property(&self, principal: &Principal, property: &str) -> Option<RateLimitStatus> {
        let key = RateLimitKey::new(*principal, property.to_string());
        self.rate_limits.get(&key).map(|entry| {
            // Get max requests from custom config, property config, or default
            let max_requests = if let Some(config) = &entry.config {
                config.max_requests_per_window
            } else if let Some(prop_config) = self.property_configs.get(&property.to_string()) {
                // For status reporting, we'll use the registered limit as a default
                prop_config.max_requests_per_window_registered
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

    pub fn reset_rate_limit_with_property(&mut self, principal: &Principal, property: &str) {
        let key = RateLimitKey::new(*principal, property.to_string());
        self.rate_limits.remove(&key);
    }

    pub fn reset_all_principal_rate_limits(&mut self, principal: &Principal) {
        // Remove all entries for a principal across all properties
        let keys_to_remove: Vec<RateLimitKey> = self.rate_limits
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

    pub fn set_principal_property_rate_limit(&mut self, principal: &Principal, property: &str, config: RateLimitConfig) {
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

    pub fn get_principal_property_config(&self, principal: &Principal, property: &str) -> Option<RateLimitConfig> {
        let key = RateLimitKey::new(*principal, property.to_string());
        self.rate_limits
            .get(&key)
            .and_then(|entry| entry.config.clone())
    }

    pub fn set_property_config(&mut self, config: SharedPropertyRateLimitConfig) {
        self.property_configs.insert(config.property.clone(), PropertyRateLimitConfig(config));
    }

    pub fn get_property_config(&self, property: &str) -> Option<SharedPropertyRateLimitConfig> {
        self.property_configs.get(&property.to_string()).map(|c| c.0.clone())
    }

    pub fn remove_property_config(&mut self, property: &str) {
        self.property_configs.remove(&property.to_string());
    }

    pub fn get_all_property_configs(&self) -> Vec<SharedPropertyRateLimitConfig> {
        self.property_configs.iter().map(|(_, v)| v.0.clone()).collect()
    }
}
