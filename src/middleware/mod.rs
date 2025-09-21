// Middleware module
pub mod security;
pub mod logging;

// Re-export commonly used middleware
pub use security::{cors_layer, security_headers, cache_control};
pub use logging::{
    request_logging, error_handling, request_id_middleware
};