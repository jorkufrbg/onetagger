use anyhow::Error;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use regex::Regex;
use crate::UrlInfo;

/// Get URL information for a given URL
pub fn get_url_info(url: &str) -> Result<UrlInfo, Error> {
    get_url_info_with_confidence(url, 0.75)
}

/// Get URL information for a given URL with a specified confidence threshold
/// The confidence parameter is used for Shazam identification when downloading songs
pub fn get_url_info_with_confidence(url: &str, _confidence: f32) -> Result<UrlInfo, Error> {
    if url.contains("youtube.com") || url.contains("youtu.be") {
        return get_youtube_info(url);
    } else if url.contains("spotify.com") {
        return get_spotify_info(url);
    } else if url.contains("soundcloud.com") {
        return get_soundcloud_info(url);
    }
    
    bail!("Unsupported URL type")
}

/// Get information from a YouTube URL
fn get_youtube_info(url: &str) -> Result<UrlInfo, Error> {
    let mut url_to_fetch = url.to_string();
    let mut channel_name = "Unknown".to_string();

    if url.contains("/@") {
        let parts: Vec<&str> = url.split("/@").collect();
        if parts.len() > 1 {
            channel_name = parts[1].split('/').next().unwrap_or("Unknown").to_string();
            if !url.ends_with("/videos") {
                url_to_fetch = format!("{}/videos", url.trim_end_matches('/'));
            }
        }
    }

    let content_type = if url.contains("/watch?v=") {
        "Video"
    } else if url.contains("/playlist?list=") {
        "Playlist"
    } else if url.contains("/@") || url.contains("/channel/") || url.contains("/user/") {
        "Channel"
    } else {
        "Content"
    };

    println!("Step 1: Preparing to fetch data from URL: {}", url_to_fetch);
    
    // For Channel type, use browser-based approach to get video count and list
    let (video_count, videos) = if content_type == "Channel" {
        match get_youtube_channel_info(&url_to_fetch) {
            Ok((count, videos)) => (count, videos),
            Err(e) => {
                println!("Error fetching channel info: {}", e);
                (0, Vec::new())
            }
        }
    } else {
        (0, Vec::new())
    };

    let description = match content_type {
        "Channel" => Some(format!("Found {} videos for channel @{}", video_count, channel_name)),
        "Playlist" => Some("Downloading all videos from this playlist".to_string()),
        _ => None,
    };

    let mut result = UrlInfo::new("youtube", content_type, &channel_name, description);
    result = result.with_url(url_to_fetch);
    
    if !videos.is_empty() {
        result = result.with_videos(videos);
    }

    Ok(result)
}

/// Get YouTube channel information using direct HTTP request approach
fn get_youtube_channel_info(videos_url: &str) -> Result<(u32, Vec<(String, String)>), Error> {
    println!("Step 2: Sending HTTP request to fetch the YouTube page");
    
    // Create a client with a user agent that mimics a browser
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;
    
    // Send the request to get the HTML content
    println!("Step 3: Waiting for response from YouTube");
    let response = client.get(videos_url).send()?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to fetch YouTube page: HTTP {}", response.status()));
    }
    
    // Get the HTML content
    let html = response.text()?;
    println!("Step 4: Parsing HTML content to extract video information");
    
    // Parse the HTML
    let document = Html::parse_document(&html);
    
    // Extract video count using regex
    let video_count = extract_video_count_from_html(&html);
    println!("Step 5: Found {} videos for this channel", video_count);
    
    // Extract video information
    let videos = extract_videos_from_html(&document);
    println!("Step 6: Extracted {} video details", videos.len());
    
    // Print video details
    for (i, (title, url)) in videos.iter().enumerate() {
        println!("Video {}: {} - {}", i + 1, title, url);
    }
    
    Ok((video_count, videos))
}

/// Extract video count from HTML content using regex
fn extract_video_count_from_html(html: &str) -> u32 {
    // Look for patterns like "XX videos" in the HTML
    let video_count_patterns = [
        r#"(\d+) videos"#,
        r#"(\d+) video"#,
        r#"videoCount":"(\d+)"#,
        r#"videosCountText":{"runs":\[{"text":"(\d+)"}\]}"#,
    ];
    
    for pattern in video_count_patterns {
        if let Some(captures) = Regex::new(pattern).unwrap().captures(html) {
            if let Some(count_str) = captures.get(1) {
                if let Ok(count) = count_str.as_str().parse::<u32>() {
                    return count;
                }
            }
        }
    }
    
    0 // Default if no count found
}

/// Extract videos from HTML document
fn extract_videos_from_html(document: &Html) -> Vec<(String, String)> {
    let mut videos = Vec::new();
    
    // Try different selectors for video titles and URLs
    let title_selectors = [
        Selector::parse("a#video-title").unwrap(),
        Selector::parse("a.yt-simple-endpoint").unwrap(),
        Selector::parse("h3.ytd-grid-video-renderer").unwrap(),
    ];
    
    for selector in &title_selectors {
        for element in document.select(selector) {
            // Extract title
            let title = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
            
            // Extract URL
            if let Some(href) = element.value().attr("href") {
                let url = if href.starts_with("http") {
                    href.to_string()
                } else if href.starts_with("/watch") {
                    format!("https://www.youtube.com{}", href)
                } else {
                    continue;
                };
                
                if !title.is_empty() && !url.is_empty() {
                    videos.push((title, url));
                }
            }
        }
        
        // If we found videos with this selector, no need to try others
        if !videos.is_empty() {
            break;
        }
    }
    
    // Alternative approach: extract from JSON data in the HTML
    if videos.is_empty() {
        extract_videos_from_json_data(document).into_iter().for_each(|v| videos.push(v));
    }
    
    videos
}

/// Extract videos from JSON data embedded in the HTML
fn extract_videos_from_json_data(document: &Html) -> Vec<(String, String)> {
    let mut videos = Vec::new();
    
    // Look for script tags that might contain JSON data
    let script_selector = Selector::parse("script").unwrap();
    
    for script in document.select(&script_selector) {
        let content = script.inner_html();
        
        // Look for patterns that might contain video data
        if content.contains("videoRenderer") || content.contains("gridVideoRenderer") {
            // Extract video titles and URLs using regex
            let title_pattern = r#""title":\s*\{\s*"runs":\s*\[\s*\{\s*"text":\s*"([^"]+)"#;
            let url_pattern = r#""videoId":\s*"([^"]+)"#;
            
            let title_regex = Regex::new(title_pattern).unwrap();
            let url_regex = Regex::new(url_pattern).unwrap();
            
            let mut titles = Vec::new();
            let mut urls = Vec::new();
            
            for cap in title_regex.captures_iter(&content) {
                if let Some(title_match) = cap.get(1) {
                    titles.push(title_match.as_str().to_string());
                }
            }
            
            for cap in url_regex.captures_iter(&content) {
                if let Some(id_match) = cap.get(1) {
                    urls.push(format!("https://www.youtube.com/watch?v={}", id_match.as_str()));
                }
            }
            
            // Match titles with URLs (assuming they're in the same order)
            let count = std::cmp::min(titles.len(), urls.len());
            for i in 0..count {
                videos.push((titles[i].clone(), urls[i].clone()));
            }
            
            // If we found videos, no need to check other scripts
            if !videos.is_empty() {
                break;
            }
        }
    }
    
    videos
}


/// Get the description of a YouTube video and extract the tracklist
fn get_youtube_video_tracklist(_client: &Client, _video_url: &str) -> Result<(String, Vec<String>), Error> {
    // For demonstration purposes, we'll use a sample description with a tracklist
    let description = r#"Chill House Mix - Amii Watson B2B Jimmi Harvey

Tracklist:
00:00 Artist One - Track One
05:30 Artist Two - Track Two
10:45 Artist Three - Track Three
15:20 Artist Four - Track Four
20:10 Artist Five - Track Five
25:30 Artist Six - Track Six
30:15 Artist Seven - Track Seven
35:40 Artist Eight - Track Eight
"#.to_string();
    
    // Extract tracklist using regex
    let tracklist = extract_tracklist_from_description(&description);
    
    Ok((description, tracklist))
}

/// Extract tracklist from video description using regex
fn extract_tracklist_from_description(description: &str) -> Vec<String> {
    // Split the description into lines
    let lines = description.lines();
    let mut tracklist = Vec::new();
    
    // Process each line
    for line in lines {
        // Look for timestamp pattern followed by artist - title
        if let Some(cap) = Regex::new(r"(?:\d+[:.)]?\s*)?(?:\d{1,2}:)?\d{1,2}:\d{2}\s*(.+?)\s*-\s*(.+)").unwrap().captures(line) {
            if cap.len() >= 3 {
                let artist = cap[1].trim();
                let title = cap[2].trim();
                
                // Clean up the extracted songs
                let cleaned_artist = Regex::new(r"^\s*(?:\d+[:.)]?\s*)?(?:\d{1,2}:)?\d{1,2}:\d{2}\s*")
                    .unwrap()
                    .replace_all(artist, "")
                    .to_string();
                    
                let cleaned_title = Regex::new(r"@\w+")
                    .unwrap()
                    .replace_all(title, "")
                    .to_string();
                    
                tracklist.push(format!("{} - {}", cleaned_artist.trim(), cleaned_title.trim()));
            }
        }
    }
    
    tracklist
}

/// Get information from a Spotify URL
fn get_spotify_info(url: &str) -> Result<UrlInfo, Error> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:85.0) Gecko/20100101 Firefox/85.0")
        .build()?;
    
    let response = client.get(url).send()?;
    let html = response.text()?;
    let document = Html::parse_document(&html);
    
    // Determine content type
    let content_type = if url.contains("/track/") {
        "Track"
    } else if url.contains("/album/") {
        "Album"
    } else if url.contains("/playlist/") {
        "Playlist"
    } else if url.contains("/artist/") {
        "Artist"
    } else {
        "Content"
    };
    
    // Extract title
    let title_selector = Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|element| element.inner_html())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    // Clean up title (remove " - Spotify" suffix)
    let title = title.replace(" - Spotify", "");
    
    // Create description
    let description = match content_type {
        "Track" => Some("Downloading single track".to_string()),
        "Album" => Some("Downloading all tracks from this album".to_string()),
        "Playlist" => Some("Downloading all tracks from this playlist".to_string()),
        "Artist" => Some("Downloading tracks from this artist".to_string()),
        _ => None
    };
    
    Ok(UrlInfo::new("spotify", content_type, &title, description))
}

/// Get information from a SoundCloud URL
fn get_soundcloud_info(url: &str) -> Result<UrlInfo, Error> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:85.0) Gecko/20100101 Firefox/85.0")
        .build()?;
    
    let response = client.get(url).send()?;
    let html = response.text()?;
    let document = Html::parse_document(&html);
    
    // Determine content type
    let content_type = if url.split('/').filter(|s| !s.is_empty()).count() <= 3 {
        "Artist"
    } else if url.contains("/sets/") {
        "Playlist"
    } else {
        "Track"
    };
    
    // Extract title
    let title_selector = Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|element| element.inner_html())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    // Clean up title
    let title = title.replace(" | Free Listening on SoundCloud", "");
    
    // Create description
    let description = match content_type {
        "Track" => Some("Downloading single track".to_string()),
        "Playlist" => Some("Downloading all tracks from this playlist".to_string()),
        "Artist" => Some("Downloading all tracks from this artist".to_string()),
        _ => None
    };
    
    Ok(UrlInfo::new("soundcloud", content_type, &title, description))
}
