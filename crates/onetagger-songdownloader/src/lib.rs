#[macro_use] extern crate anyhow;

use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;

mod url_info;

pub use url_info::*;

/// Main struct for the SongDownloader functionality
pub struct SongDownloader {
    client: Client
}

impl SongDownloader {
    /// Create a new instance
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:85.0) Gecko/20100101 Firefox/85.0")
            .build()
            .unwrap();

        SongDownloader {
            client
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlInfo {
    pub platform: String,
    pub content_type: String,
    pub title: String,
    pub description: Option<String>,
}

impl UrlInfo {
    pub fn new(platform: &str, content_type: &str, title: &str, description: Option<String>) -> Self {
        UrlInfo {
            platform: platform.to_string(),
            content_type: content_type.to_string(),
            title: title.to_string(),
            description,
        }
    }
} 