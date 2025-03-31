#[macro_use] extern crate anyhow;

use serde::{Serialize, Deserialize};

mod url_info;

pub use url_info::{get_url_info, get_url_info_with_confidence};

/// Main struct for the SongDownloader functionality
pub struct SongDownloader {
    // Remove unused client field
}

impl SongDownloader {
    /// Create a new instance
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlInfo {
    pub platform: String,
    pub content_type: String,
    pub title: String,
    pub description: Option<String>,
    pub video_tracklists: Option<std::collections::HashMap<String, Vec<String>>>,
    pub videos: Option<Vec<(String, String, Vec<String>)>>,
    pub url: String,
}

impl UrlInfo {
    pub fn new(platform: &str, content_type: &str, title: &str, description: Option<String>) -> Self {
        UrlInfo {
            platform: platform.to_string(),
            content_type: content_type.to_string(),
            title: title.to_string(),
            description,
            video_tracklists: None,
            videos: None,
            url: String::new(),
        }
    }
    
    pub fn with_tracklists(mut self, tracklists: std::collections::HashMap<String, Vec<String>>) -> Self {
        self.video_tracklists = Some(tracklists);
        self
    }
    
    pub fn with_videos(mut self, videos: Vec<(String, String, Vec<String>)>) -> Self {
        self.videos = Some(videos);
        self
    }
    
    pub fn with_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }
    
    pub fn add_tracklist(mut self, video_title: String, tracklist: Vec<String>) -> Self {
        let mut tracklists = self.video_tracklists.unwrap_or_default();
        tracklists.insert(video_title, tracklist);
        self.video_tracklists = Some(tracklists);
        self
    }
}
