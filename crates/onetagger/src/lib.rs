use onetagger_shared::songsdownloader::{AnalyzeSongsRequest, DownloadSongsRequest, analyze_songs, download_songs};

pub mod songsdownloader {
    use serde::{Deserialize, Serialize};
    use anyhow::Result;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AnalyzeSongsRequest {
        pub url: String,
        pub confidence: f32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct DownloadSongsRequest {
        pub url: String,
        pub output_path: String,
        pub confidence: f32,
        pub enable_auto_tag: bool,
        pub auto_tag_config: Option<String>,
        pub enable_audio_features: bool,
        pub songs: Vec<FoundSong>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FoundSong {
        pub title: String,
        pub artist: String,
        pub video_url: String,
        pub timestamp: Option<u64>,
    }

    pub async fn analyze_songs(request: AnalyzeSongsRequest) -> Result<Vec<FoundSong>> {
        // For now, return an empty vector
        // TODO: Implement actual song analysis
        Ok(Vec::new())
    }

    pub async fn download_songs(request: DownloadSongsRequest) -> Result<()> {
        // For now, just return Ok
        // TODO: Implement actual song download
        Ok(())
    }
} 