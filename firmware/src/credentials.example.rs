//! WiFi credentials template. Copy to `credentials.rs` and fill in:
//!
//!     cp src/credentials.example.rs src/credentials.rs

// Known networks are tried in order. Use an empty password for an open network.
pub const WIFI_NETWORKS: &[(&str, &str)] = &[
    ("your-ssid", "your-password"),
    ("backup-ssid", "backup-password"),
];
