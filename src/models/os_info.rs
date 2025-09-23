// Operating System Information Model
// Represents comprehensive OS details for system identification and troubleshooting

use serde::{Deserialize, Serialize};
use std::fmt;

/// Operating system information structure
/// Contains static OS details that don't change during runtime
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OsInfo {
    /// Operating system name (e.g., "Linux", "macOS", "Windows")
    pub name: String,
    /// OS version (e.g., "22.04", "13.5", "11")
    pub version: String,
    /// System architecture (e.g., "x86_64", "aarch64")
    pub architecture: String,
    /// Kernel version (e.g., "5.15.0-89-generic")
    pub kernel_version: String,
    /// Distribution name for Linux (e.g., "Ubuntu", "CentOS")
    /// None for non-Linux operating systems
    pub distribution: Option<String>,
    /// Long OS description/pretty name
    pub long_description: String,
}

/// Validation errors specific to OS information
#[derive(Debug, Clone, PartialEq)]
pub enum OsInfoValidationError {
    /// OS name is empty or invalid
    InvalidName { name: String },
    /// OS version is empty or invalid
    InvalidVersion { version: String },
    /// Architecture is empty or invalid
    InvalidArchitecture { architecture: String },
    /// Kernel version is empty or invalid
    InvalidKernelVersion { kernel_version: String },
    /// Distribution is empty when provided (Some(""))
    InvalidDistribution { distribution: String },
    /// Long description is empty or invalid
    InvalidLongDescription { description: String },
}

impl fmt::Display for OsInfoValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OsInfoValidationError::InvalidName { name } => {
                write!(f, "Invalid OS name: '{}'", name)
            }
            OsInfoValidationError::InvalidVersion { version } => {
                write!(f, "Invalid OS version: '{}'", version)
            }
            OsInfoValidationError::InvalidArchitecture { architecture } => {
                write!(f, "Invalid architecture: '{}'", architecture)
            }
            OsInfoValidationError::InvalidKernelVersion { kernel_version } => {
                write!(f, "Invalid kernel version: '{}'", kernel_version)
            }
            OsInfoValidationError::InvalidDistribution { distribution } => {
                write!(f, "Invalid distribution: '{}'", distribution)
            }
            OsInfoValidationError::InvalidLongDescription { description } => {
                write!(f, "Invalid long description: '{}'", description)
            }
        }
    }
}

impl std::error::Error for OsInfoValidationError {}

impl OsInfo {
    /// Validate the OS information structure
    /// Returns an error if any field is invalid
    pub fn validate(&self) -> Result<(), OsInfoValidationError> {
        // Validate name is non-empty
        if self.name.trim().is_empty() {
            return Err(OsInfoValidationError::InvalidName {
                name: self.name.clone(),
            });
        }

        // Validate version is non-empty
        if self.version.trim().is_empty() {
            return Err(OsInfoValidationError::InvalidVersion {
                version: self.version.clone(),
            });
        }

        // Validate architecture is non-empty
        if self.architecture.trim().is_empty() {
            return Err(OsInfoValidationError::InvalidArchitecture {
                architecture: self.architecture.clone(),
            });
        }

        // Validate kernel version is non-empty
        if self.kernel_version.trim().is_empty() {
            return Err(OsInfoValidationError::InvalidKernelVersion {
                kernel_version: self.kernel_version.clone(),
            });
        }

        // Validate distribution is non-empty if provided
        if let Some(ref dist) = self.distribution {
            if dist.trim().is_empty() {
                return Err(OsInfoValidationError::InvalidDistribution {
                    distribution: dist.clone(),
                });
            }
        }

        // Validate long description is non-empty
        if self.long_description.trim().is_empty() {
            return Err(OsInfoValidationError::InvalidLongDescription {
                description: self.long_description.clone(),
            });
        }

        Ok(())
    }

    /// Create a fallback OsInfo instance when detection fails
    /// Used when sysinfo cannot determine OS details
    pub fn fallback() -> Self {
        Self {
            name: "Unknown".to_string(),
            version: "Unknown".to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            kernel_version: "Unknown".to_string(),
            distribution: None,
            long_description: "Operating system information unavailable".to_string(),
        }
    }

    /// Check if this instance uses fallback values
    /// Useful for logging and debugging
    pub fn is_fallback(&self) -> bool {
        self.name == "Unknown" 
            && self.version == "Unknown" 
            && self.kernel_version == "Unknown"
    }
}

impl Default for OsInfo {
    fn default() -> Self {
        Self::fallback()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_os_info() {
        let os_info = OsInfo {
            name: "Linux".to_string(),
            version: "22.04".to_string(),
            architecture: "x86_64".to_string(),
            kernel_version: "5.15.0-89-generic".to_string(),
            distribution: Some("Ubuntu".to_string()),
            long_description: "Ubuntu 22.04.3 LTS".to_string(),
        };

        assert!(os_info.validate().is_ok());
        assert!(!os_info.is_fallback());
    }

    #[test]
    fn test_fallback_creation() {
        let fallback = OsInfo::fallback();
        
        assert_eq!(fallback.name, "Unknown");
        assert_eq!(fallback.version, "Unknown");
        assert_eq!(fallback.architecture, std::env::consts::ARCH);
        assert_eq!(fallback.kernel_version, "Unknown");
        assert_eq!(fallback.distribution, None);
        assert_eq!(fallback.long_description, "Operating system information unavailable");
        assert!(fallback.is_fallback());
        assert!(fallback.validate().is_ok());
    }

    #[test]
    fn test_empty_name_validation() {
        let os_info = OsInfo {
            name: "".to_string(),
            version: "22.04".to_string(),
            architecture: "x86_64".to_string(),
            kernel_version: "5.15.0-89-generic".to_string(),
            distribution: Some("Ubuntu".to_string()),
            long_description: "Ubuntu 22.04.3 LTS".to_string(),
        };

        let result = os_info.validate();
        assert!(result.is_err());
        matches!(result.unwrap_err(), OsInfoValidationError::InvalidName { .. });
    }

    #[test]
    fn test_empty_distribution_validation() {
        let os_info = OsInfo {
            name: "Linux".to_string(),
            version: "22.04".to_string(),
            architecture: "x86_64".to_string(),
            kernel_version: "5.15.0-89-generic".to_string(),
            distribution: Some("".to_string()), // Empty distribution
            long_description: "Ubuntu 22.04.3 LTS".to_string(),
        };

        let result = os_info.validate();
        assert!(result.is_err());
        matches!(result.unwrap_err(), OsInfoValidationError::InvalidDistribution { .. });
    }

    #[test]
    fn test_none_distribution_validation() {
        let os_info = OsInfo {
            name: "macOS".to_string(),
            version: "13.5".to_string(),
            architecture: "aarch64".to_string(),
            kernel_version: "22.6.0".to_string(),
            distribution: None, // None is valid for non-Linux
            long_description: "macOS Ventura 13.5".to_string(),
        };

        assert!(os_info.validate().is_ok());
    }

    #[test]
    fn test_serialization() {
        let os_info = OsInfo {
            name: "Windows".to_string(),
            version: "11".to_string(),
            architecture: "x86_64".to_string(),
            kernel_version: "10.0.22621".to_string(),
            distribution: None,
            long_description: "Windows 11 Pro".to_string(),
        };

        let json = serde_json::to_string(&os_info).unwrap();
        let deserialized: OsInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(os_info, deserialized);
    }
}