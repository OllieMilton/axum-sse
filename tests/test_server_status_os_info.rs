// Contract test for GET /api/server-status with OS info
// This test validates the API contract including OS information structure
// TEST MUST FAIL until OsInfo is implemented

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::Value;

#[tokio::test]
async fn test_server_status_includes_os_info() {
    // Create test server
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();

    // Test GET /api/server-status endpoint
    let response = server.get("/api/server-status").await;
    
    // Verify response structure
    response.assert_status(StatusCode::OK);
    let json: Value = response.json();
    
    // Verify top-level structure
    assert!(json["success"].as_bool().unwrap_or(false), "Response should indicate success");
    assert!(json["data"].is_object(), "Response should contain data object");
    
    let data = &json["data"];
    assert!(data["server_info"].is_object(), "Data should contain server_info object");
    
    let server_info = &data["server_info"];
    
    // Verify existing server_info fields
    assert!(server_info["hostname"].is_string(), "server_info should contain hostname");
    assert!(server_info["version"].is_string(), "server_info should contain version");
    assert!(server_info["start_time"].is_string(), "server_info should contain start_time");
    assert!(server_info["environment"].is_string(), "server_info should contain environment");
    
    // Verify OS info structure (THIS WILL FAIL until implemented)
    assert!(server_info["os_info"].is_object(), "server_info should contain os_info object");
    
    let os_info = &server_info["os_info"];
    
    // Verify all required OS info fields
    assert!(os_info["name"].is_string(), "os_info should contain name field");
    assert!(os_info["version"].is_string(), "os_info should contain version field");
    assert!(os_info["architecture"].is_string(), "os_info should contain architecture field");
    assert!(os_info["kernel_version"].is_string(), "os_info should contain kernel_version field");
    assert!(os_info["long_description"].is_string(), "os_info should contain long_description field");
    
    // distribution field is optional but should be null or string
    let distribution = &os_info["distribution"];
    assert!(
        distribution.is_null() || distribution.is_string(),
        "os_info distribution should be null or string"
    );
    
    // Verify field values are non-empty strings
    let name = os_info["name"].as_str().unwrap();
    assert!(!name.is_empty(), "OS name should not be empty");
    
    let version = os_info["version"].as_str().unwrap();
    assert!(!version.is_empty(), "OS version should not be empty");
    
    let architecture = os_info["architecture"].as_str().unwrap();
    assert!(!architecture.is_empty(), "OS architecture should not be empty");
    
    let kernel_version = os_info["kernel_version"].as_str().unwrap();
    assert!(!kernel_version.is_empty(), "Kernel version should not be empty");
    
    let long_description = os_info["long_description"].as_str().unwrap();
    assert!(!long_description.is_empty(), "OS long description should not be empty");
}

#[tokio::test]
async fn test_os_info_contract_validation() {
    // Create test server
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/server-status").await;
    response.assert_status(StatusCode::OK);
    
    let json: Value = response.json();
    let os_info = &json["data"]["server_info"]["os_info"];
    
    // Contract validation: ensure OS info matches expected schema
    // This validates the JSON schema requirements from contracts/server-status-api.md
    
    // Required fields must exist and be strings
    let required_string_fields = ["name", "version", "architecture", "kernel_version", "long_description"];
    for field in required_string_fields {
        assert!(
            os_info[field].is_string(),
            "Required field '{}' must be a string, got: {:?}",
            field,
            os_info[field]
        );
        
        let value = os_info[field].as_str().unwrap();
        assert!(
            !value.is_empty(),
            "Required field '{}' must not be empty",
            field
        );
    }
    
    // distribution is optional but if present must be non-empty string
    if let Some(distribution) = os_info["distribution"].as_str() {
        assert!(!distribution.is_empty(), "If distribution is present, it must not be empty");
    }
    
    // Validate that OS name contains recognizable values
    let name = os_info["name"].as_str().unwrap();
    let valid_os_names = ["Linux", "Windows", "macOS", "FreeBSD", "Unknown"];
    assert!(
        valid_os_names.iter().any(|&valid_name| name.contains(valid_name)),
        "OS name '{}' should contain a recognizable OS identifier",
        name
    );
}