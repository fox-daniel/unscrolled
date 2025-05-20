// shared/src/lib.rs

pub mod api; // Add this line

pub mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct Message {
        pub content: String,
        pub timestamp: String,
    }
}
