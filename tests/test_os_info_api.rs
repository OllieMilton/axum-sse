// Integration tests for OS information collection and API integration
// These tests validate the OS collection service and API endpoint behavior
// Tests MUST FAIL until OS collection is implemented

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::Value;
use std::time::Instant;

#[tokio::test]
async fn test_api_integration_with_os_info() {
    // Test that the API properly integrates OS information into responses
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/server-status").await;
    response.assert_status(StatusCode::OK);
    
    let json: Value = response.json();
    let os_info = &json["data"]["server_info"]["os_info"];
    
    // Verify OS collection integration (WILL FAIL until implemented)
    assert!(os_info.is_object(), "OS info should be collected and included");
    
    // Verify that OS information is properly collected from the system
    let name = os_info["name"].as_str().unwrap();
    assert!(!name.is_empty(), "OS name should be collected from system");
    
    // On Linux, we should see specific patterns
    #[cfg(target_os = "linux")]
    {
        assert!(name.to_lowercase().contains("linux"), "Should detect Linux OS");
    }
    
    // On macOS, we should see specific patterns
    #[cfg(target_os = "macos")]
    {
        assert!(name.to_lowercase().contains("mac"), "Should detect macOS");
    }
    
    // Verify architecture is detected
    let arch = os_info["architecture"].as_str().unwrap();
    let valid_architectures = ["x86_64", "aarch64", "arm64", "i686"];
    assert!(
        valid_architectures.iter().any(|&a| arch.contains(a)),
        "Architecture '{}' should be valid",
        arch
    );
}

#[tokio::test]
async fn test_os_info_static_loading() {
    // Test that OS info is loaded once and remains consistent
    // This validates FR-007: OS info loaded once on page initialization
    
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    // Make multiple requests
    let response1 = server.get("/api/server-status").await;
    let response2 = server.get("/api/server-status").await;
    let response3 = server.get("/api/server-status").await;
    
    // All responses should be successful
    response1.assert_status(StatusCode::OK);
    response2.assert_status(StatusCode::OK);
    response3.assert_status(StatusCode::OK);
    
    let json1: Value = response1.json();
    let json2: Value = response2.json();
    let json3: Value = response3.json();
    
    let os_info1 = &json1["data"]["server_info"]["os_info"];
    let os_info2 = &json2["data"]["server_info"]["os_info"];
    let os_info3 = &json3["data"]["server_info"]["os_info"];
    
    // OS info should be identical across requests (static loading)
    assert_eq!(os_info1, os_info2, "OS info should be consistent between requests");
    assert_eq!(os_info2, os_info3, "OS info should be consistent between requests");
    
    // Verify specific fields are identical
    assert_eq!(
        os_info1["name"].as_str(),
        os_info2["name"].as_str(),
        "OS name should be consistent"
    );
    assert_eq!(
        os_info1["version"].as_str(),
        os_info2["version"].as_str(),
        "OS version should be consistent"
    );
    assert_eq!(
        os_info1["architecture"].as_str(),
        os_info2["architecture"].as_str(),
        "Architecture should be consistent"
    );
}

#[tokio::test]
async fn test_os_info_performance() {
    // Test that OS info doesn't impact API response time significantly
    // Should maintain <200ms response time requirement
    
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let start = Instant::now();
    let response = server.get("/api/server-status").await;
    let duration = start.elapsed();
    
    response.assert_status(StatusCode::OK);
    
    // Verify response time is reasonable (allowing for test overhead)
    assert!(
        duration.as_millis() < 500,
        "API response with OS info should be fast, took {}ms",
        duration.as_millis()
    );
    
    // Verify OS info is present in fast response
    let json: Value = response.json();
    assert!(
        json["data"]["server_info"]["os_info"].is_object(),
        "Fast response should still include OS info"
    );
}

#[tokio::test]
async fn test_cross_platform_os_detection() {
    // Test that OS detection works appropriately for the current platform
    // Validates that we get platform-specific information
    
    let app = axum_sse::create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/api/server-status").await;
    response.assert_status(StatusCode::OK);
    
    let json: Value = response.json();
    let os_info = &json["data"]["server_info"]["os_info"];
    
    let name = os_info["name"].as_str().unwrap();
    let distribution = os_info["distribution"].as_str();
    
    // Platform-specific validations
    if name.contains("Linux") {
        // On Linux, we should have distribution info
        if let Some(dist) = distribution {
            assert!(!dist.is_empty(), "Linux distribution should not be empty");
            
            // Common Linux distributions
            let common_distros = ["Ubuntu", "Debian", "CentOS", "RedHat", "Fedora", "SUSE", "Arch"];
            let has_known_distro = common_distros.iter().any(|&distro| dist.contains(distro));
            
            if !has_known_distro {
                println!("Unknown Linux distribution detected: {}", dist);
            }
        }
        
        // Linux should have meaningful kernel version
        let kernel = os_info["kernel_version"].as_str().unwrap();
        assert!(
            kernel.chars().any(|c| c.is_ascii_digit()),
            "Linux kernel version should contain version numbers"
        );
    } else if name.contains("Windows") {
        // Windows should not have distribution
        assert!(distribution.is_none(), "Windows should not have distribution field");
        
        // Windows should have NT kernel info
        let kernel = os_info["kernel_version"].as_str().unwrap();
        // Windows kernel info might be different format
        assert!(!kernel.is_empty(), "Windows should have kernel version info");
    } else if name.contains("macOS") || name.contains("Darwin") {
        // macOS should not have distribution
        assert!(distribution.is_none(), "macOS should not have distribution field");
        
        // Darwin kernel version
        let kernel = os_info["kernel_version"].as_str().unwrap();
        assert!(!kernel.is_empty(), "macOS should have Darwin kernel version");
    }
}