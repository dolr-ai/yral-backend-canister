# Video Generation Tracking in Rate Limits Canister

**Date**: 2025-07-29  
**Type**: Feature Implementation  
**Component**: Rate Limits Canister

## Summary

Implemented video generation request tracking within the rate limits canister to support asynchronous video generation workflows with rate limiting, status tracking, and result polling.

## Implementation Details

### 1. Data Model Extensions

#### New Types (in `shared_utils`)
```rust
// Composite key for efficient user-based queries
pub struct VideoGenRequestKey {
    pub principal: Principal,
    pub counter: u64,
}

// Request status tracking
pub enum VideoGenRequestStatus {
    Pending,
    Processing,
    Complete(String), // Result URL
    Failed(String),   // Error message
}

// Request details
pub struct VideoGenRequest {
    pub model_name: String,
    pub prompt: String,
    pub status: VideoGenRequestStatus,
    pub created_at: u64,
    pub updated_at: u64,
}
```

#### Memory Management
- Memory ID 4: `video_gen_requests` - StableBTreeMap<VideoGenRequestKey, VideoGenRequest>
- Memory ID 5: `user_request_counters` - StableBTreeMap<Principal, u64>

### 2. Key Design Decisions

#### Composite Key Structure
- Used `(Principal, counter)` instead of UUID for request IDs
- Benefits:
  - Natural partitioning by user
  - Efficient range queries for user's requests
  - No UUID generation overhead
  - Sequential ordering maintained

#### Counter-Based Request IDs
- Each user has an auto-incrementing counter
- Starts at 1, increments with each request
- Enables efficient cursor-based pagination

#### Integration with Existing Rate Limits
- Video generation requests check rate limits before creation
- Uses property-based rate limiting (e.g., "VIDEOGEN")
- Respects registered/unregistered user distinctions

### 3. API Endpoints

```rust
// Create request (admin/offchain agent only)
create_video_generation_request(
    principal: Principal,
    model_name: String,
    prompt: String,
    property: String,
    is_registered: bool
) -> Result<VideoGenRequestKey, String>

// Update status (admin/offchain agent only)
update_video_generation_status(
    key: VideoGenRequestKey,
    status: VideoGenRequestStatus
) -> Result<(), String>

// Query endpoints
get_video_generation_request(key: VideoGenRequestKey) -> Option<VideoGenRequest>
get_user_video_generation_requests(
    principal: Principal,
    start: Option<u64>,  // Cursor
    limit: Option<u64>   // Default 10, max 100
) -> Vec<(VideoGenRequestKey, VideoGenRequest)>
poll_video_generation_status(key: VideoGenRequestKey) -> Result<VideoGenRequestStatus, String>
```

### 4. Testing Insights

#### Rate Limit Configuration
- Default rate limits are restrictive (1 request per 24 hours for registered users)
- Tests require setting higher limits for video generation property
- Example test configuration:
  ```rust
  set_property_rate_limit_config(
      "VIDEOGEN",
      10u64,  // max_requests_per_window_registered
      5u64,   // max_requests_per_window_unregistered
      60u64,  // window_duration_seconds
  )
  ```

#### Test Coverage
- Request creation with auto-incrementing counters
- Rate limit enforcement
- Status transitions (Pending → Processing → Complete/Failed)
- Cursor-based pagination
- Error handling for non-existent requests

### 5. Cursor-Based Pagination Implementation

```rust
// Start from most recent if no cursor provided
let start_counter = start.unwrap_or(max_counter);

// Calculate range going backwards
let end_counter = if start_counter > limit {
    start_counter - limit + 1
} else {
    1
};

// Iterate backwards for newest-first ordering
for counter in (end_counter..=start_counter).rev() {
    // Fetch and add to results
}
```

## Lessons Learned

### 1. Canister Memory Management
- IC stable structures require careful memory ID management
- Each new data structure needs a unique memory ID
- StableBTreeMap provides efficient key-value storage with persistence

### 2. Rate Limit Integration
- Property-based rate limiting provides flexibility
- Different limits for registered/unregistered users
- Rate limit checks must happen before any state changes

### 3. Testing Considerations
- Integration tests need proper rate limit configuration
- Default configurations may be too restrictive for testing
- Service canister provisioning required for full integration tests

### 4. Type Safety with Candid
- All types must implement CandidType for IC compatibility
- Enums with data (like VideoGenRequestStatus) work well with Candid
- Auto-generation of .did files saves manual updates

## Future Improvements

1. **Secondary Indices**: Consider adding principal → Vec<request_id> mapping for more efficient user queries
2. **Cleanup Mechanism**: Implement periodic cleanup of old completed/failed requests
3. **Metrics**: Add request count metrics per model type
4. **Batch Operations**: Support batch status updates for efficiency

## Integration Points

This implementation enables the full video generation flow:
1. Client → Offchain Agent (request)
2. Offchain Agent → Rate Limit Canister (check & create placeholder)
3. Offchain Agent → Qstash (queue job)
4. Qstash → Model → Offchain Agent (callback)
5. Offchain Agent → Rate Limit Canister (update status)
6. Client → Rate Limit Canister (poll for result)

## Code References

- Types: `src/lib/shared_utils/src/canister_specific/rate_limits/types.rs:140-201`
- Memory: `src/canister/rate_limits/src/data_model/memory.rs:11-12,39-45`
- Data Model: `src/canister/rate_limits/src/data_model/mod.rs:390-475`
- API: `src/canister/rate_limits/src/api/video_gen_tracking.rs`
- Tests: `src/lib/integration_tests/tests/rate_limits/test_video_gen_tracking.rs`