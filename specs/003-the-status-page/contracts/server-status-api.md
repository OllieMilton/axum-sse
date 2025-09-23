# API Contract: Server Status with OS Information

## Endpoint: GET /api/server-status

### Description
Returns current server metrics and static information including comprehensive OS details.

### Request
```http
GET /api/server-status HTTP/1.1
Host: localhost:3000
Accept: application/json
```

### Response (Success)
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "success": true,
  "data": {
    "server_metrics": {
      "timestamp": "2025-09-22T10:30:00Z",
      "memory_usage": {
        "total_bytes": 16777216000,
        "used_bytes": 8388608000,
        "available_bytes": 8388608000,
        "percentage": 50.0
      },
      "cpu_usage": {
        "system_percentage": 15.5,
        "user_percentage": 25.3,
        "total_percentage": 40.8
      },
      "uptime": {
        "secs": 86400,
        "nanos": 0
      },
      "network_metrics": {
        "interfaces": [
          {
            "name": "eth0",
            "bytes_received": 1048576,
            "bytes_transmitted": 524288,
            "bytes_received_per_sec": 1024,
            "bytes_transmitted_per_sec": 512
          }
        ]
      }
    },
    "collection_interval_seconds": 5,
    "server_info": {
      "hostname": "server01",
      "version": "0.1.0",
      "start_time": "2025-09-22T09:30:00Z",
      "environment": "development",
      "os_info": {
        "name": "Linux",
        "version": "22.04",
        "architecture": "x86_64",
        "kernel_version": "5.15.0-89-generic",
        "distribution": "Ubuntu",
        "long_description": "Ubuntu 22.04.3 LTS"
      }
    }
  }
}
```

### Response (Error)
```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{
  "success": false,
  "error": {
    "code": "METRICS_COLLECTION_FAILED",
    "message": "Failed to collect system metrics",
    "details": "OS information unavailable"
  }
}
```

## Schema Definitions

### OsInfo Schema
```json
{
  "type": "object",
  "required": ["name", "version", "architecture", "kernel_version", "long_description"],
  "properties": {
    "name": {
      "type": "string",
      "description": "Operating system name",
      "examples": ["Linux", "Windows", "macOS", "FreeBSD"]
    },
    "version": {
      "type": "string", 
      "description": "OS version identifier",
      "examples": ["22.04", "11", "13.5"]
    },
    "architecture": {
      "type": "string",
      "description": "System architecture",
      "examples": ["x86_64", "aarch64", "i686"]
    },
    "kernel_version": {
      "type": "string",
      "description": "Kernel version string",
      "examples": ["5.15.0-89-generic", "22.6.0", "13.5.0"]
    },
    "distribution": {
      "type": ["string", "null"],
      "description": "Linux distribution name (null for non-Linux)",
      "examples": ["Ubuntu", "CentOS", "Debian", null]
    },
    "long_description": {
      "type": "string",
      "description": "Human-readable OS description",
      "examples": ["Ubuntu 22.04.3 LTS", "Windows 11 Pro", "macOS Ventura 13.5"]
    }
  }
}
```

### Updated ServerInfo Schema
```json
{
  "type": "object",
  "required": ["hostname", "version", "start_time", "environment", "os_info"],
  "properties": {
    "hostname": {
      "type": "string",
      "description": "Server hostname"
    },
    "version": {
      "type": "string", 
      "description": "Application version"
    },
    "start_time": {
      "type": "string",
      "format": "date-time",
      "description": "Server start timestamp"
    },
    "environment": {
      "type": "string",
      "enum": ["development", "staging", "production"],
      "description": "Deployment environment"
    },
    "os_info": {
      "$ref": "#/definitions/OsInfo"
    }
  }
}
```

## Backward Compatibility
- Existing API consumers continue to work
- New `os_info` field added to `server_info` object
- No breaking changes to existing response structure
- API version remains unchanged (additive change only)

## Performance Requirements
- Response time: <200ms (unchanged from existing endpoint)
- OS information cached at startup, no runtime collection overhead
- Memory impact: <1KB additional per response