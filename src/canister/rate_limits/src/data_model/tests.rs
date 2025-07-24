#[cfg(test)]
mod tests {
    use crate::CanisterData;
    use candid::Principal;

    #[test]
    fn test_blacklist_blocks_rate_limits() {
        let mut data = CanisterData::default();
        let principal = Principal::from_text("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let property = "test_property";
        
        // Before blacklisting, should not be rate limited (no existing entries)
        assert!(!data.is_rate_limited_with_property(&principal, property, false));
        
        // Add property to blacklist
        data.add_to_blacklist(property.to_string());
        
        // After blacklisting, should be rate limited
        assert!(data.is_rate_limited_with_property(&principal, property, false));
        
        // Remove from blacklist
        data.remove_from_blacklist(property);
        
        // Should not be rate limited again
        assert!(!data.is_rate_limited_with_property(&principal, property, false));
    }
    
    #[test]
    fn test_blacklist_all_blocks_all_properties() {
        let mut data = CanisterData::default();
        let principal = Principal::from_text("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        
        // Test different properties
        let properties = vec!["prop1", "prop2", "default"];
        
        // Before blacklisting "all", no properties should be rate limited
        for prop in &properties {
            assert!(!data.is_rate_limited_with_property(&principal, prop, false));
        }
        
        // Add "all" to blacklist
        data.add_to_blacklist("all".to_string());
        
        // After blacklisting "all", all properties should be rate limited
        for prop in &properties {
            assert!(data.is_rate_limited_with_property(&principal, prop, false));
        }
        
        // Clear blacklist
        data.clear_blacklist();
        
        // Should not be rate limited again
        for prop in &properties {
            assert!(!data.is_rate_limited_with_property(&principal, prop, false));
        }
    }
}