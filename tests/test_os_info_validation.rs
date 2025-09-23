// Unit tests for OsInfo validation
// Tests OsInfo struct creation, validation, and error handling
// TEST MUST FAIL until OsInfo model is implemented

use axum_sse::models::os_info::{OsInfo, OsInfoValidationError};
use axum_sse::models::status_data::StatusValidationError;

#[test]
fn test_os_info_valid_creation() {
    // Test creating valid OsInfo instances
    // This will fail until OsInfo struct is defined
    
    let os_info = OsInfo {
        name: "Linux".to_string(),
        version: "22.04".to_string(),
        architecture: "x86_64".to_string(),
        kernel_version: "5.15.0-89-generic".to_string(),
        distribution: Some("Ubuntu".to_string()),
        long_description: "Ubuntu 22.04.3 LTS".to_string(),
    };
    
    // Validation should pass for valid data
    assert!(os_info.validate().is_ok(), "Valid OS info should pass validation");
}

#[test]
fn test_os_info_validation_empty_name() {
    // Test validation fails for empty OS name
    
    let os_info = OsInfo {
        name: "".to_string(), // Empty name should fail
        version: "22.04".to_string(),
        architecture: "x86_64".to_string(),
        kernel_version: "5.15.0-89-generic".to_string(),
        distribution: Some("Ubuntu".to_string()),
        long_description: "Ubuntu 22.04.3 LTS".to_string(),
    };
    
    let result = os_info.validate();
    assert!(result.is_err(), "Empty OS name should fail validation");
    
    match result.unwrap_err() {
        OsInfoValidationError::InvalidName { name } => {
            assert_eq!(name, "", "Error should contain the invalid name");
        }
        _ => panic!("Should get InvalidName error for empty name"),
    }
}

#[test]
fn test_os_info_validation_empty_version() {
    // Test validation fails for empty version
    
    let os_info = OsInfo {
        name: "Linux".to_string(),
        version: "".to_string(), // Empty version should fail
        architecture: "x86_64".to_string(),
        kernel_version: "5.15.0-89-generic".to_string(),
        distribution: Some("Ubuntu".to_string()),
        long_description: "Ubuntu 22.04.3 LTS".to_string(),
    };
    
    let result = os_info.validate();
    assert!(result.is_err(), "Empty OS version should fail validation");
    
    match result.unwrap_err() {
        OsInfoValidationError::InvalidVersion { version } => {
            assert_eq!(version, "", "Error should contain the invalid version");
        }
        _ => panic!("Should get InvalidVersion error for empty version"),
    }
}

#[test]
fn test_os_info_validation_empty_architecture() {
    // Test validation fails for empty architecture
    
    let os_info = OsInfo {
        name: "Linux".to_string(),
        version: "22.04".to_string(),
        architecture: "".to_string(), // Empty architecture should fail
        kernel_version: "5.15.0-89-generic".to_string(),
        distribution: Some("Ubuntu".to_string()),
        long_description: "Ubuntu 22.04.3 LTS".to_string(),
    };
    
    let result = os_info.validate();
    assert!(result.is_err(), "Empty architecture should fail validation");
    
    match result.unwrap_err() {
        OsInfoValidationError::InvalidArchitecture { architecture } => {
            assert_eq!(architecture, "", "Error should contain the invalid architecture");
        }
        _ => panic!("Should get InvalidArchitecture error for empty architecture"),
    }
}

#[test]
fn test_os_info_validation_empty_kernel_version() {
    // Test validation fails for empty kernel version
    
    let os_info = OsInfo {
        name: "Linux".to_string(),
        version: "22.04".to_string(),
        architecture: "x86_64".to_string(),
        kernel_version: "".to_string(), // Empty kernel version should fail
        distribution: Some("Ubuntu".to_string()),
        long_description: "Ubuntu 22.04.3 LTS".to_string(),
    };
    
    let result = os_info.validate();
    assert!(result.is_err(), "Empty kernel version should fail validation");
    
    match result.unwrap_err() {
        OsInfoValidationError::InvalidKernelVersion { kernel_version } => {
            assert_eq!(kernel_version, "", "Error should contain the invalid kernel version");
        }
        _ => panic!("Should get InvalidKernelVersion error for empty kernel version"),
    }
}

#[test]
fn test_os_info_validation_empty_description() {
    // Test validation fails for empty long description
    
    let os_info = OsInfo {
        name: "Linux".to_string(),
        version: "22.04".to_string(),
        architecture: "x86_64".to_string(),
        kernel_version: "5.15.0-89-generic".to_string(),
        distribution: Some("Ubuntu".to_string()),
        long_description: "".to_string(), // Empty description should fail
    };
    
    let result = os_info.validate();
    assert!(result.is_err(), "Empty long description should fail validation");
    
    match result.unwrap_err() {
        OsInfoValidationError::InvalidLongDescription { description } => {
            assert_eq!(description, "", "Error should contain the invalid description");
        }
        _ => panic!("Should get InvalidLongDescription error for empty description"),
    }
}

#[test]
fn test_os_info_validation_empty_distribution() {
    // Test validation fails for empty distribution (if present)
    
    let os_info = OsInfo {
        name: "Linux".to_string(),
        version: "22.04".to_string(),
        architecture: "x86_64".to_string(),
        kernel_version: "5.15.0-89-generic".to_string(),
        distribution: Some("".to_string()), // Empty distribution should fail
        long_description: "Ubuntu 22.04.3 LTS".to_string(),
    };
    
    let result = os_info.validate();
    assert!(result.is_err(), "Empty distribution string should fail validation");
    
    match result.unwrap_err() {
        OsInfoValidationError::InvalidDistribution { distribution } => {
            assert_eq!(distribution, "", "Error should contain the invalid distribution");
        }
        _ => panic!("Should get InvalidDistribution error for empty distribution"),
    }
}

#[test]
fn test_os_info_validation_none_distribution() {
    // Test validation passes for None distribution (valid for non-Linux)
    
    let os_info = OsInfo {
        name: "Windows".to_string(),
        version: "11".to_string(),
        architecture: "x86_64".to_string(),
        kernel_version: "10.0.22621".to_string(),
        distribution: None, // None distribution should be valid
        long_description: "Windows 11 Pro".to_string(),
    };
    
    let result = os_info.validate();
    assert!(result.is_ok(), "None distribution should be valid for non-Linux systems");
}

#[test]
fn test_os_info_serialization() {
    // Test that OsInfo can be serialized/deserialized for API responses
    
    let os_info = OsInfo {
        name: "macOS".to_string(),
        version: "13.5".to_string(),
        architecture: "aarch64".to_string(),
        kernel_version: "22.6.0".to_string(),
        distribution: None,
        long_description: "macOS Ventura 13.5".to_string(),
    };
    
    // Test serialization to JSON
    let json = serde_json::to_string(&os_info)
        .expect("OsInfo should be serializable to JSON");
    
    // Test deserialization from JSON  
    let deserialized: OsInfo = serde_json::from_str(&json)
        .expect("OsInfo should be deserializable from JSON");
    
    // Verify round-trip integrity
    assert_eq!(os_info.name, deserialized.name);
    assert_eq!(os_info.version, deserialized.version);
    assert_eq!(os_info.architecture, deserialized.architecture);
    assert_eq!(os_info.kernel_version, deserialized.kernel_version);
    assert_eq!(os_info.distribution, deserialized.distribution);
    assert_eq!(os_info.long_description, deserialized.long_description);
}

#[test]
fn test_status_validation_error_integration() {
    // Test that OsInfo validation errors integrate with StatusValidationError
    // This tests the error handling flow from status_data.rs
    
    // This will test the From trait implementation once it's added
    let os_error = OsInfoValidationError::InvalidName { 
        name: "".to_string() 
    };
    
    let status_error: StatusValidationError = os_error.into();
    
    // Verify the error message is properly formatted
    let error_message = status_error.to_string();
    assert!(
        error_message.contains("OS") || error_message.contains("name"),
        "Status error should contain OS-related information: {}",
        error_message
    );
}