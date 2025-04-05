#[macro_use] extern crate anyhow;

use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

mod query_url;
mod download_songs;

pub use query_url::{get_query_url, get_query_url_with_confidence};
pub use download_songs::{download_songs, generate_output_file, SongInfo};

/// Main struct for the SongDownloader functionality
pub struct SongDownloader {
    // Configuration fields
    pub url: Option<String>,
    pub directory: Option<PathBuf>,
    pub confidence: f32,
    pub output_format: String,
}

impl SongDownloader {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            url: None,
            directory: None,
            confidence: 0.75,
            output_format: "csv".to_string(),
        }
    }
    
    /// Set the URL to query
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }
    
    /// Set the output directory
    pub fn with_directory(mut self, directory: &Path) -> Self {
        self.directory = Some(directory.to_path_buf());
        self
    }
    
    /// Set the confidence threshold for matching
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
    
    /// Set the output format (csv or json)
    pub fn with_output_format(mut self, format: &str) -> Self {
        self.output_format = format.to_string();
        self
    }
    
    /// Query a URL and generate output file
    pub fn query_url(&self) -> Result<PathBuf, anyhow::Error> {
        let url = self.url.as_ref().ok_or_else(|| anyhow::anyhow!("URL is required"))?;
        
        // Get URL information
        let url_info = get_query_url_with_confidence(url, self.confidence)?;
        
        // Create output directory if it doesn't exist
        let output_dir = PathBuf::from("query-url-output");
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }
        
        // Generate output file
        generate_output_file(&url_info, &output_dir, self.directory.as_deref(), &self.output_format)
    }
    
    /// Download songs from a CSV/JSON file
    pub fn download_songs(&self, csv_path: &Path) -> Result<(), anyhow::Error> {
        let directory = self.directory.as_ref().ok_or_else(|| anyhow::anyhow!("Directory is required"))?;
        download_songs(csv_path, directory)
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

/// Get URL information for a given URL with default confidence
pub fn get_url_info(url: &str) -> Result<UrlInfo, anyhow::Error> {
    get_query_url(url)
}

/// Get URL information for a given URL with specified confidence
pub fn get_url_info_with_confidence(url: &str, confidence: f32) -> Result<UrlInfo, anyhow::Error> {
    get_query_url_with_confidence(url, confidence)
}

/// Download songs from a CSV/JSON file to a directory
pub fn download_songs_from_file(csv_path: &Path, directory: &Path) -> Result<(), anyhow::Error> {
    download_songs(csv_path, directory)
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
