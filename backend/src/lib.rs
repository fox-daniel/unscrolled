//! Placeholder backend library
//! This file exists to satisfy Cargo's requirements for a valid crate

// Re-export shared models for convenience
pub use shared::models;

/// Placeholder function to ensure the crate compiles
pub fn health_check() -> &'static str {
    "Backend placeholder is working"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(health_check(), "Backend placeholder is working");
    }
}
