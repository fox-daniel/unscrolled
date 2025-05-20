// shared/src/api.rs
pub struct ApiEndpoints {
    pub base_url: &'static str,
}

impl ApiEndpoints {
    pub fn new(base_url: &'static str) -> Self {
        Self { base_url }
    }

    pub fn messages_endpoint(&self) -> String {
        format!("{}/api/messages", self.base_url)
    }
}

pub const LOCAL_API_URL: &str = "http://127.0.0.1:8000";
pub const PROD_API_URL: &str = "https://api.unscrolled.com";

// Convenient function to get the appropriate API URL based on compile-time environment
#[cfg(debug_assertions)]
pub fn get_api_base_url() -> &'static str {
    LOCAL_API_URL
}

#[cfg(not(debug_assertions))]
pub fn get_api_base_url() -> &'static str {
    PROD_API_URL
}
