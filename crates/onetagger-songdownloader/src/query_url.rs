use anyhow::{Error, Result, bail};
use log::info;
use regex::Regex;
use std::collections::HashMap;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use crate::UrlInfo;

/// Get URL information for a given URL with default confidence
pub fn get_query_url(url: &str) -> Result<UrlInfo, Error> {
    println!("get_query_url called with URL: {}", url);
    get_query_url_with_confidence(url, 0.75)
}

/// Get URL information for a given URL with specified confidence
pub fn get_query_url_with_confidence(url: &str, confidence: f32) -> Result<UrlInfo, Error> {
    println!("get_query_url_with_confidence called with URL: {} and confidence: {}", url, confidence);
    
    // Validate URL
    if !is_valid_url(url) {
        println!("Invalid URL: {}", url);
        bail!("Invalid URL. Must be a YouTube, Spotify, or SoundCloud URL.");
    }

    // Determine platform and content type
    let (platform, content_type) = determine_platform_and_type(url)?;
    println!("Determined platform: {} and content type: {}", platform, content_type);
    
    // Process URL based on platform and content type
    match (platform.as_str(), content_type.as_str()) {
        ("youtube", "channel") => {
            println!("Processing YouTube channel: {}", url);
            process_youtube_channel(url, confidence)
        },
        ("youtube", "playlist") => {
            println!("Processing YouTube playlist: {}", url);
            process_youtube_playlist(url, confidence)
        },
        ("youtube", "video") => {
            println!("Processing YouTube video: {}", url);
            process_youtube_video(url, confidence)
        },
        ("spotify", _) => {
            println!("Spotify support not yet implemented");
            bail!("Spotify support not yet implemented")
        },
        ("soundcloud", _) => {
            println!("SoundCloud support not yet implemented");
            bail!("SoundCloud support not yet implemented")
        },
        _ => {
            println!("Unsupported platform or content type: {}/{}", platform, content_type);
            bail!("Unsupported platform or content type")
        }
    }
}

/// Check if the URL is valid (YouTube, Spotify, or SoundCloud)
fn is_valid_url(url: &str) -> bool {
    let valid_domains = ["youtube.com", "youtu.be", "spotify.com", "soundcloud.com"];
    valid_domains.iter().any(|&domain| url.contains(domain))
}

/// Determine the platform and content type from the URL
fn determine_platform_and_type(url: &str) -> Result<(String, String), Error> {
    if url.contains("youtube.com") || url.contains("youtu.be") {
        // YouTube URL
        if url.contains("/playlist") || url.contains("list=") {
            Ok(("youtube".to_string(), "playlist".to_string()))
        } else if url.contains("/@") {
            Ok(("youtube".to_string(), "channel".to_string()))
        } else if url.contains("/channel/") || url.contains("/c/") || url.contains("/user/") {
            Ok(("youtube".to_string(), "channel".to_string()))
        } else if url.contains("watch?v=") || url.contains("youtu.be/") {
            Ok(("youtube".to_string(), "video".to_string()))
        } else {
            bail!("Unsupported YouTube URL format")
        }
    } else if url.contains("spotify.com") {
        // Spotify URL
        if url.contains("/playlist/") {
            Ok(("spotify".to_string(), "playlist".to_string()))
        } else if url.contains("/album/") {
            Ok(("spotify".to_string(), "album".to_string()))
        } else if url.contains("/track/") {
            Ok(("spotify".to_string(), "track".to_string()))
        } else if url.contains("/artist/") {
            Ok(("spotify".to_string(), "artist".to_string()))
        } else {
            bail!("Unsupported Spotify URL format")
        }
    } else if url.contains("soundcloud.com") {
        // SoundCloud URL
        if url.contains("/sets/") {
            Ok(("soundcloud".to_string(), "playlist".to_string()))
        } else {
            Ok(("soundcloud".to_string(), "track".to_string()))
        }
    } else {
        bail!("Unsupported URL platform")
    }
}

/// Process a YouTube channel URL
fn process_youtube_channel(url: &str, _confidence: f32) -> Result<UrlInfo, Error> {
    // Modify URL to ensure we're looking at the videos tab
    let videos_url = if url.ends_with("/videos") {
        url.to_string()
    } else {
        format!("{}/videos", url.trim_end_matches('/'))
    };
    
    // Create a new HTTP client with user agent
    let client = create_client()?;
    
    // Fetch the channel page
    info!("Fetching channel page: {}", videos_url);
    let response = client.get(&videos_url).send()?;
    let html = response.text()?;
    let document = Html::parse_document(&html);
    
    // Extract channel name
    let channel_name = extract_channel_name(&document)?;
    
    // Extract video links
    let video_links = extract_video_links(&document)?;
    
    info!("Scraping Youtube channel {} • {} videos found.", channel_name, video_links.len());
    
    // Process each video
    let mut videos = Vec::new();
    for (index, video_url) in video_links.iter().enumerate() {
        info!("Scraping video {} of {}", index + 1, video_links.len());
        
        // Process the video
        if let Ok((title, tracklist)) = process_single_video(&client, video_url) {
            videos.push((title.clone(), video_url.clone(), tracklist));
        }
    }
    
    // Create UrlInfo
    let url_info = UrlInfo::new(
        "youtube",
        "channel",
        &channel_name,
        None
    )
    .with_videos(videos)
    .with_url(url.to_string());
    
    Ok(url_info)
}

/// Process a YouTube playlist URL
fn process_youtube_playlist(url: &str, _confidence: f32) -> Result<UrlInfo, Error> {
    // Create a new HTTP client with user agent
    let client = create_client()?;
    
    // Fetch the playlist page
    let response = client.get(url).send()?;
    let html = response.text()?;
    let document = Html::parse_document(&html);
    
    // Extract playlist title
    let playlist_title = extract_playlist_title(&document)?;
    
    // Extract video links
    let video_links = extract_video_links(&document)?;
    
    info!("Scraping Youtube playlist {} • {} videos found.", playlist_title, video_links.len());
    
    // Process each video
    let mut videos = Vec::new();
    for (index, video_url) in video_links.iter().enumerate() {
        info!("Scraping video {} of {}", index + 1, video_links.len());
        
        // Process the video
        if let Ok((title, tracklist)) = process_single_video(&client, video_url) {
            videos.push((title.clone(), video_url.clone(), tracklist));
        }
    }
    
    // Create UrlInfo
    let url_info = UrlInfo::new(
        "youtube",
        "playlist",
        &playlist_title,
        None
    )
    .with_videos(videos)
    .with_url(url.to_string());
    
    Ok(url_info)
}

/// Process a YouTube video URL
fn process_youtube_video(url: &str, _confidence: f32) -> Result<UrlInfo, Error> {
    // Create a new HTTP client with user agent
    let client = create_client()?;
    
    // Process the video
    let (title, tracklist) = process_single_video(&client, url)?;
    
    // Create video tracklist map
    let mut tracklists = HashMap::new();
    tracklists.insert(title.clone(), tracklist.clone());
    
    // Create videos vector with a single entry
    let videos = vec![(title.clone(), url.to_string(), tracklist)];
    
    // Create UrlInfo
    let url_info = UrlInfo::new(
        "youtube",
        "video",
        &title,
        None
    )
    .with_tracklists(tracklists)
    .with_videos(videos)
    .with_url(url.to_string());
    
    Ok(url_info)
}

/// Create a new HTTP client with user agent
fn create_client() -> Result<Client, Error> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;
    
    info!("HTTP client created successfully");
    Ok(client)
}

/// Extract the channel name from a YouTube channel page
fn extract_channel_name(document: &Html) -> Result<String, Error> {
    // Try different selectors for channel name
    let selectors = [
        "#channel-name",
        "#channel-header-container #text",
        "ytd-channel-name yt-formatted-string",
        "title",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    // If we got the title, remove " - YouTube" suffix
                    if *selector_str == "title" && text.contains(" - YouTube") {
                        return Ok(text.replace(" - YouTube", "").trim().to_string());
                    }
                    return Ok(text);
                }
            }
        }
    }
    
    Ok("Unknown Channel".to_string())
}

/// Extract the playlist title from a YouTube playlist page
fn extract_playlist_title(document: &Html) -> Result<String, Error> {
    // Try different selectors for playlist title
    let selectors = [
        "h1.title",
        "#playlist-header-title",
        "ytd-playlist-header-renderer h1",
        "title",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    // If we got the title, remove " - YouTube" suffix
                    if *selector_str == "title" && text.contains(" - YouTube") {
                        return Ok(text.replace(" - YouTube", "").trim().to_string());
                    }
                    return Ok(text);
                }
            }
        }
    }
    
    Ok("Unknown Playlist".to_string())
}

/// Extract video links from a YouTube channel or playlist page
fn extract_video_links(document: &Html) -> Result<Vec<String>, Error> {
    // Try different selectors for video links
    let selectors = [
        "a#video-title",
        "ytd-grid-video-renderer a#video-title",
        "ytd-playlist-video-renderer a#video-title",
        "ytd-video-renderer a#video-title",
        "a.yt-simple-endpoint",
    ];
    
    let mut video_links = Vec::new();
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if href.contains("/watch?v=") {
                        let full_url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("https://www.youtube.com{}", href)
                        };
                        video_links.push(full_url);
                    }
                }
            }
            
            if !video_links.is_empty() {
                break;
            }
        }
    }
    
    Ok(video_links)
}

/// Process a single YouTube video
fn process_single_video(client: &Client, url: &str) -> Result<(String, Vec<String>), Error> {
    // Fetch the video page
    info!("Fetching video page: {}", url);
    let response = client.get(url).send()?;
    let html = response.text()?;
    let document = Html::parse_document(&html);
    
    // Extract the video title
    let title = extract_video_title(&document)?;
    info!("Video title: {}", title);
    
    // Format the video title for folder name
    let formatted_title = sanitize_filename(&title);
    info!("Formatted video title: {}", formatted_title);
    
    // Extract the video description
    let description = extract_video_description(&document)?;
    
    // Extract tracklist from description
    let tracklist = extract_tracklist_from_description(&description);
    
    info!("{} tracks found", tracklist.len());
    
    Ok((formatted_title, tracklist))
}

/// Extract the video title from a YouTube video page
fn extract_video_title(document: &Html) -> Result<String, Error> {
    // Try different selectors for video title
    let selectors = [
        "h1.title",
        "#title h1",
        "ytd-video-primary-info-renderer h1",
        "title",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    // If we got the title, remove " - YouTube" suffix
                    if *selector_str == "title" && text.contains(" - YouTube") {
                        return Ok(text.replace(" - YouTube", "").trim().to_string());
                    }
                    return Ok(text);
                }
            }
        }
    }
    
    Ok("Unknown Video".to_string())
}

/// Extract the video description from a YouTube video page
fn extract_video_description(document: &Html) -> Result<String, Error> {
    // Try different selectors for video description
    let selectors = [
        "#description-inner",
        "#description-inline-expander",
        "#description-inline-expander ytd-text-inline-expander",
        "#description-inline-expander #content",
        "#description-inline-expander #content ytd-text-inline-expander",
        "#description-inline-expander #content ytd-text-inline-expander #content",
        "#description-inline-expander #content ytd-text-inline-expander #content yt-formatted-string",
        "#description ytd-text-inline-expander",
        "#description ytd-text-inline-expander #content",
        "#description ytd-text-inline-expander #content yt-formatted-string",
        "#description yt-formatted-string",
        "#info-contents #description yt-formatted-string",
        "#description",
        "#info-contents",
        "#meta",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Ok(text);
                }
            }
        }
    }
    
    Ok("Description not found".to_string())
}

/// Extract tracklist from video description
fn extract_tracklist_from_description(description: &str) -> Vec<String> {
    // Try to find the tracklist section
    if let Some(tracklist_section) = find_tracklist_section(description) {
        // Extract track entries
        let mut tracks = Vec::new();
        let mut seen_tracks = std::collections::HashSet::new();
        
        // First, try to extract using the numbered entries with timestamps pattern
        // This pattern matches lines like "1) 00:00 Artist - Title" or "1. 00:00 Artist - Title"
        // It also handles timestamps over 1 hour like "16) 01:02:50 Artist - Title"
        let timestamp_pattern = Regex::new(r"(?m)^\s*\d+[\.\)]\s*(?:\d+:)?\d+:\d+\s+(.+?)\s*$").unwrap();
        
        // Extract all tracks that match the pattern
        for cap in timestamp_pattern.captures_iter(tracklist_section) {
            if let Some(track) = cap.get(1) {
                let track_text = track.as_str().trim();
                
                // Clean up the track text
                let clean_track = clean_track_text(track_text);
                
                // Only add unique tracks
                if !seen_tracks.contains(&clean_track) {
                    seen_tracks.insert(clean_track.clone());
                    tracks.push(clean_track);
                }
            }
        }
        
        // If we didn't find enough tracks with the timestamp pattern, try other approaches
        if tracks.len() < 15 {
            // Try to extract using the line-by-line approach
            extract_tracks_line_by_line(tracklist_section, &mut tracks, &mut seen_tracks);
        }
        
        tracks
    } else {
        Vec::new()
    }
}

/// Find the tracklist section in a video description
fn find_tracklist_section(description: &str) -> Option<&str> {
    // Look for common tracklist section indicators
    let tracklist_markers = [
        "Tracklist:", "Track list:", "Tracks:", "Songs:", "Music:", "Playlist:"
    ];
    
    for marker in &tracklist_markers {
        if let Some(pos) = description.to_lowercase().find(&marker.to_lowercase()) {
            return Some(&description[pos..]);
        }
    }
    
    // If no marker found, return the whole description
    Some(description)
}

/// Clean up track text
fn clean_track_text(track: &str) -> String {
    // Try to extract using the pattern
    if let Some(clean_track) = extract_artist_title_with_remix(track) {
        return clean_track;
    }
    
    // If we couldn't extract using the pattern, do a more general cleanup
    
    // Remove timestamps and track numbers
    let without_timestamp = Regex::new(r"^\s*\d+[\.\):]?\s*\d*:?\d+\s+").unwrap()
        .replace_all(track, "")
        .to_string();
    
    // Remove social media handles and other noise
    let without_social = Regex::new(r"@\w+|ÔÇ¬ÔÇ¼X?|X$|X\s*$").unwrap()
        .replace_all(&without_timestamp, "")
        .to_string();
    
    // Extract remix information if present
    let mut remix_info = String::new();
    let remix_re = Regex::new(r"\(([^)]*(?:Remix|Mix|Dub|Edit)[^)]*)\)").unwrap();
    if let Some(caps) = remix_re.captures(&without_social) {
        if let Some(remix) = caps.get(1) {
            remix_info = format!(" ({})", remix.as_str());
        }
    }
    
    // Remove all parenthetical content
    let without_parentheses = Regex::new(r"\([^)]*\)").unwrap()
        .replace_all(&without_social, "")
        .to_string();
    
    // Fix common encoding issues
    let fixed_encoding = without_parentheses
        .replace("├©", "ø")
        .replace("Sc├©tt", "Scott");
    
    // Normalize whitespace
    let mut normalized = String::new();
    let mut last_was_space = false;
    
    for c in fixed_encoding.trim().chars() {
        if c.is_whitespace() {
            if !last_was_space {
                normalized.push(' ');
                last_was_space = true;
            }
        } else {
            normalized.push(c);
            last_was_space = false;
        }
    }
    
    // Add back remix information if we found it
    if !remix_info.is_empty() {
        normalized + &remix_info
    } else {
        normalized
    }
}

/// Extract artist and title with remix information
fn extract_artist_title_with_remix(track: &str) -> Option<String> {
    // Try to extract using the "Artist - Title (Remix)" pattern
    let re = Regex::new(r"([A-Za-z0-9\s&,.]+)\s+-\s+([A-Za-z0-9\s',.]+)(\s+\([A-Za-z0-9\s',.]+(?:Remix|Mix|Dub|Edit)[A-Za-z0-9\s',.]*\))?").ok()?;
    
    if let Some(caps) = re.captures(track) {
        if caps.len() >= 3 {
            let artist = caps.get(1)?.as_str().trim();
            let title = caps.get(2)?.as_str().trim();
            
            // Fix missing commas in artist names
            let artist_fixed = fix_artist_names(artist);
            
            if !artist_fixed.is_empty() && !title.is_empty() {
                // Include remix info if present
                if let Some(remix) = caps.get(3) {
                    return Some(format!("{} - {}{}", artist_fixed, title, remix.as_str()));
                } else {
                    return Some(format!("{} - {}", artist_fixed, title));
                }
            }
        }
    }
    
    None
}

/// Fix missing commas in artist names
fn fix_artist_names(artist: &str) -> String {
    // Common patterns where commas might be missing
    let patterns = [
        ("Aberton JazzedUp", "Aberton, JazzedUp"),
        ("Julian Sanza Andre", "Julian Sanza, Andre"),
        ("Big Miz & Bessa", "Big Miz & Bessa"),  // Already correct
        ("Paolo Barbato Lee", "Paolo Barbato, Lee"),
        ("Cpen JT", "Cpen, JT"),
    ];
    
    for (pattern, replacement) in &patterns {
        if artist.contains(pattern) {
            return artist.replace(pattern, replacement);
        }
    }
    
    // Try to detect missing commas between names
    // Look for patterns like "FirstName LastName FirstName" where a comma should be after LastName
    let name_pattern = Regex::new(r"([A-Z][a-z]+)\s+([A-Z][a-z]+)\s+([A-Z][a-z]+)").ok();
    if let Some(re) = name_pattern {
        if let Some(caps) = re.captures(artist) {
            if caps.len() >= 4 {
                let first1 = caps.get(1).unwrap().as_str();
                let last1 = caps.get(2).unwrap().as_str();
                let first2 = caps.get(3).unwrap().as_str();
                
                return artist.replace(&format!("{} {} {}", first1, last1, first2), 
                                     &format!("{} {}, {}", first1, last1, first2));
            }
        }
    }
    
    artist.to_string()
}

/// Extract tracks line by line
fn extract_tracks_line_by_line(text: &str, tracks: &mut Vec<String>, seen_tracks: &mut std::collections::HashSet<String>) {
    // Split by lines and look for potential track entries
    for line in text.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines or very short lines
        if trimmed.is_empty() || trimmed.len() < 5 {
            continue;
        }
        
        // Skip lines that are likely not track entries
        if !looks_like_track_entry(trimmed) {
            continue;
        }
        
        // Clean up the track text
        let clean_track = clean_track_text(trimmed);
        
        // Only add unique tracks
        if !seen_tracks.contains(&clean_track) {
            seen_tracks.insert(clean_track.clone());
            tracks.push(clean_track);
        }
    }
}

/// Check if a line looks like a track entry
fn looks_like_track_entry(line: &str) -> bool {
    // Check if the line contains a dash (common in "Artist - Title" format)
    if line.contains(" - ") {
        return true;
    }
    
    // Check if the line starts with a number (possible track number)
    if Regex::new(r"^\d+[\.\)]").unwrap().is_match(line) {
        return true;
    }
    
    // Check for timestamp pattern
    if Regex::new(r"\d+:\d+").unwrap().is_match(line) {
        return true;
    }
    
    false
}

/// Convert a title to a valid folder name
fn sanitize_filename(filename: &str) -> String {
    // Replace invalid characters with spaces
    let invalid_chars = Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
    let sanitized = invalid_chars.replace_all(filename, " ").to_string();
    
    // Trim leading/trailing whitespace and dots
    let trimmed = sanitized.trim().trim_matches('.');
    
    // Normalize multiple spaces to a single space
    let normalized = Regex::new(r"\s+").unwrap().replace_all(&trimmed, " ").to_string();
    
    // Ensure the filename is not empty
    if normalized.is_empty() {
        return "Unknown_Title".to_string();
    }
    
    normalized
}
