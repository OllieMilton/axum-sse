# Data Model: OS Information Extension

## Overview
Extend existing `ServerInfo` structure to include comprehensive operating system information. This maintains architectural consistency while adding the required OS details.

## Entity Extensions

### ServerInfo (Extended)
**Purpose**: Static server identification and configuration including OS details  
**Location**: `src/models/status_data.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    // Existing fields
    pub hostname: String,
    pub version: String,
    pub start_time: DateTime<Utc>,
    pub environment: String,
    
    // New OS information fields
    pub os_info: OsInfo,
}
```

### OsInfo (New)
**Purpose**: Comprehensive operating system information  
**Location**: `src/models/os_info.rs` (new file)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub distribution: Option<String>,
    /// Long OS description/pretty name
    pub long_description: String,
}
```

## Validation Rules

### OsInfo Validation
- **name**: Must be non-empty string, typical values: "Linux", "Windows", "macOS", "FreeBSD"
- **version**: Must be non-empty string, semantic versioning preferred
- **architecture**: Must be non-empty string, common values: "x86_64", "aarch64", "i686"
- **kernel_version**: Must be non-empty string
- **distribution**: Optional field, non-empty if present
- **long_description**: Must be non-empty string

### ServerInfo Validation (Updated)
- Existing validation rules remain unchanged
- **os_info**: Must pass OsInfo validation

## Data Flow

### Collection (Startup)
1. Application startup triggers OS information collection
2. `sysinfo` crate provides OS data via `System::name()`, `System::version()`, etc.
3. Data cached in memory for duration of application lifecycle
4. No runtime re-collection (OS info is static)

### API Response
1. `/api/server-status` endpoint includes `server_info.os_info`
2. Frontend receives OS data on page load
3. OS information displayed in dedicated UI section

## Integration Points

### Backend Integration
- Extend `MetricsService::collect_server_info()` to include OS collection
- Add OS info validation to `StatusData::validate()`
- Ensure OS collection happens once at service initialization

### Frontend Integration  
- Extend status page component to display OS information section
- Add OS info to TypeScript interfaces
- Include OS details in status page layout

## Error Handling

### Collection Errors
- If OS info cannot be determined, use fallback values:
  - `name`: "Unknown"
  - `version`: "Unknown" 
  - `architecture`: std::env::consts::ARCH
  - `kernel_version`: "Unknown"
  - `distribution`: None
  - `long_description`: "Operating system information unavailable"

### Validation Errors
- OS info validation errors included in `StatusValidationError`
- Non-blocking: application continues with fallback values
- Log warnings for debugging purposes

## Memory Impact
- Additional ~200-500 bytes per OsInfo instance
- Single instance cached for application lifetime
- Negligible memory footprint increase