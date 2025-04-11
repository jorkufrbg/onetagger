use anyhow::Error;
use log::info;
use crate::UrlInfo;

/// Process a SoundCloud URL
pub fn process_soundcloud(url: &str, confidence: f32) -> Result<UrlInfo, Error> {
    // For now, just return a skeleton implementation since we're having build issues
    // Once the build is fixed, we can implement the full version
    
    // Determine content type from URL
    let content_type = if url.contains("/sets/") {
        "playlist"
    } else {
        "track"
    };
    
    // Create a placeholder title
    let title = format!("SoundCloud {} from {}", content_type, url);
    
    // Create a simple UrlInfo with just the basics
    let url_info = UrlInfo::new(
        "soundcloud",
        content_type,
        &title,
        None
    )
    .with_url(url.to_string());
    
    Ok(url_info)
}

/// Determine the content type from a SoundCloud URL
fn determine_soundcloud_type(url: &str) -> Result<String, Error> {
    if url.contains("/sets/") {
        Ok("playlist".to_string())
    } else {
        Ok("track".to_string())
    }
}

/// Process a SoundCloud track URL
fn process_soundcloud_track(url: &str) -> Result<UrlInfo, Error> {
    info!("Fetching SoundCloud track: {}", url);
    
    // Create a client with user agent
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36")
        .build()?;
    
    // Fetch the track page
    let response = client.get(url).send()
        .context("Failed to fetch SoundCloud track page")?;
    
    let html = response.text()?;
    let document = Html::parse_document(&html);
    
    // Extract track title and artist
    let title = extract_soundcloud_title(&document)?;
    let artist = extract_soundcloud_artist(&document)?;
    
    // Format as "Artist - Title"
    let track_title = format!("{} - {}", artist, title);
    
    // Create videos vector with a single entry
    let videos = vec![(track_title.clone(), url.to_string(), Vec::<String>::new())];
    
    // Create UrlInfo
    let url_info = UrlInfo::new(
        "soundcloud",
        "track",
        &track_title,
        None
    )
    .with_videos(videos)
    .with_url(url.to_string());
    
    info!("Successfully processed SoundCloud track: {}", track_title);
    Ok(url_info)
}

/// Process a SoundCloud playlist URL
fn process_soundcloud_playlist(url: &str) -> Result<UrlInfo, Error> {
    info!("Fetching SoundCloud playlist: {}", url);
    
    // Create a client with user agent
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36")
        .build()?;
    
    // Fetch the playlist page
    let response = client.get(url).send()
        .context("Failed to fetch SoundCloud playlist page")?;
    
    let html = response.text()?;
    let document = Html::parse_document(&html);
    
    // Extract playlist title and author
    let playlist_title = extract_soundcloud_playlist_title(&document)?;
    let author = extract_soundcloud_artist(&document)?;
    
    // Extract track links from the playlist
    let track_links = extract_soundcloud_playlist_tracks(&document)?;
    
    info!("Found {} tracks in SoundCloud playlist", track_links.len());
    
    // Create videos vector for each track
    let mut videos = Vec::new();
    
    // Process each track minimally for now
    // In a full implementation, we would fetch each track page for details
    for (i, track_link) in track_links.iter().enumerate() {
        // For now, use placeholder titles based on position
        // A production implementation would fetch each track details
        let track_title = format!("Track {} from {}", i + 1, playlist_title);
        
        videos.push((track_title, track_link.clone(), Vec::<String>::new()));
    }
    
    // Format playlist title with author
    let full_title = format!("Playlist: {} by {}", playlist_title, author);
    
    // Create UrlInfo
    let url_info = UrlInfo::new(
        "soundcloud",
        "playlist",
        &full_title,
        None
    )
    .with_videos(videos)
    .with_url(url.to_string());
    
    info!("Successfully processed SoundCloud playlist: {}", full_title);
    Ok(url_info)
}

/// Extract the track title from a SoundCloud page
fn extract_soundcloud_title(document: &Html) -> Result<String, Error> {
    // Try different selectors that might contain the title
    let selectors = [
        "[itemprop='name']",
        ".soundTitle__title span",
        ".soundTitle__title",
        "h1",
        "title",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    // If we got the title, clean it if needed
                    if *selector_str == "title" {
                        // SoundCloud titles are usually in format "Title by Artist"
                        let parts: Vec<&str> = text.split(" by ").collect();
                        if parts.len() >= 1 {
                            return Ok(parts[0].trim().to_string());
                        }
                    }
                    return Ok(text);
                }
            }
        }
    }
    
    // If no title found, return a placeholder
    Ok("Unknown Track".to_string())
}

/// Extract the artist name from a SoundCloud page
fn extract_soundcloud_artist(document: &Html) -> Result<String, Error> {
    // Try different selectors that might contain the artist
    let selectors = [
        "[itemprop='author'] [itemprop='name']",
        ".soundTitle__username",
        ".soundTitle__info a",
        ".soundContext__usernameLink",
        "title",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    // If we got the title, clean it if needed
                    if *selector_str == "title" {
                        // SoundCloud titles are usually in format "Title by Artist"
                        let parts: Vec<&str> = text.split(" by ").collect();
                        if parts.len() >= 2 {
                            return Ok(parts[1].trim().to_string());
                        }
                    }
                    return Ok(text);
                }
            }
        }
    }
    
    // If no artist found, return a placeholder
    Ok("Unknown Artist".to_string())
}

/// Extract the playlist title from a SoundCloud playlist page
fn extract_soundcloud_playlist_title(document: &Html) -> Result<String, Error> {
    // Try different selectors that might contain the playlist title
    let selectors = [
        ".soundTitle__title span",
        ".playlistHeader__title",
        "h1",
        "title",
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
    
    // If no title found, return a placeholder
    Ok("Unknown Playlist".to_string())
}

/// Extract track links from a SoundCloud playlist page
fn extract_soundcloud_playlist_tracks(document: &Html) -> Result<Vec<String>, Error> {
    let mut track_links = Vec::new();
    
    // Try to find track elements in the playlist
    if let Ok(track_selector) = Selector::parse(".trackList__item") {
        for track_element in document.select(&track_selector) {
            // Find the link element in each track
            if let Ok(link_selector) = Selector::parse("a.trackItem__trackTitle") {
                if let Some(link) = track_element.select(&link_selector).next() {
                    if let Some(href) = link.value().attr("href") {
                        // Convert relative URLs to absolute
                        let full_url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("https://soundcloud.com{}", href)
                        };
                        
                        track_links.push(full_url);
                    }
                }
            }
        }
    }
    
    // If we didn't find any tracks, try a more generic link selector
    if track_links.is_empty() {
        if let Ok(link_selector) = Selector::parse("a[href*='/tracks/']") {
            for link in document.select(&link_selector) {
                if let Some(href) = link.value().attr("href") {
                    // Only include links that point to tracks
                    if href.contains("/tracks/") || href.contains("/track/") {
                        // Convert relative URLs to absolute
                        let full_url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("https://soundcloud.com{}", href)
                        };
                        
                        track_links.push(full_url);
                    }
                }
            }
        }
    }
    
    // If we found at least one track link, return them
    if !track_links.is_empty() {
        Ok(track_links)
    } else {
        // If we couldn't find any tracks, return an empty vector and log a warning
        warn!("Could not find any tracks in the SoundCloud playlist");
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_determine_soundcloud_type() {
        // Test playlist URL
        let content_type = determine_soundcloud_type("https://soundcloud.com/user-name/sets/playlist-name").unwrap();
        assert_eq!(content_type, "playlist");
        
        // Test track URL
        let content_type = determine_soundcloud_type("https://soundcloud.com/user-name/track-name").unwrap();
        assert_eq!(content_type, "track");
    }
}