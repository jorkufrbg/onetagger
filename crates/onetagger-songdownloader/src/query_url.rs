use anyhow::{Error, Result, bail, Context};
use log::{info, warn, debug};
use regex::Regex;
use std::collections::HashMap;
use reqwest::blocking::{Client, ClientBuilder};
use scraper::{Html, Selector};
use crate::UrlInfo;
use std::time::Duration;
use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;

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
    
    // Fetch the channel page with enhanced scraping
    info!("Fetching channel page: {}", videos_url);
    let document = get_youtube_page(&client, &videos_url)?;
    
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
            // Only add videos with tracklists
            if !tracklist.is_empty() {
                videos.push((title.clone(), video_url.clone(), tracklist));
            } else {
                info!("Skipping video with no tracklist: {}", title);
            }
        } else {
            info!("Failed to process video: {}", video_url);
        }
    }
    
    // Log the number of videos with tracklists found
    info!("Found {} videos with tracklists out of {} total videos", videos.len(), video_links.len());
    
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
    
    // Fetch the playlist page with enhanced scraping
    info!("Fetching playlist page: {}", url);
    let document = get_youtube_page(&client, url)?;
    
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
            // Only add videos with tracklists
            if !tracklist.is_empty() {
                videos.push((title.clone(), video_url.clone(), tracklist));
            } else {
                info!("Skipping video with no tracklist: {}", title);
            }
        } else {
            info!("Failed to process video: {}", video_url);
        }
    }
    
    // Log the number of videos with tracklists found
    info!("Found {} videos with tracklists out of {} total videos", videos.len(), video_links.len());
    
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
    
    // Process the video with enhanced scraping
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
    let client = ClientBuilder::new()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .cookie_store(true)
        .gzip(true)
        .build()?;
    
    info!("HTTP client created successfully");
    Ok(client)
}

/// Uses JavaScript to extract more complex details from a YouTube page
/// Returns the HTML content after JavaScript processing
fn extract_with_javascript(url: &str) -> Result<String, Error> {
    debug!("Extracting with JavaScript: {}", url);
    
    // Create a temporary JavaScript file
    let mut js_file = NamedTempFile::new()?;
    
    // First check if puppeteer is installed
    let check_puppeteer = Command::new("node")
        .args(["-e", "try { require('puppeteer'); console.log('puppeteer-installed'); } catch(e) { console.log('puppeteer-missing'); }"])
        .output()
        .context("Failed to execute Node.js. Make sure it's installed and in your PATH")?;
    
    let check_result = String::from_utf8_lossy(&check_puppeteer.stdout).trim().to_string();
    
    if check_result != "puppeteer-installed" {
        info!("Puppeteer is not installed. Using fallback method without JavaScript extraction.");
        return Ok(String::new());
    }
    
    let js_code = format!(r#"
    const puppeteer = require('puppeteer');

    (async () => {{
        try {{
            const browser = await puppeteer.launch({{
                headless: true,
                args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage']
            }});
            const page = await browser.newPage();
            
            // Set viewport and user agent
            await page.setViewport({{ width: 1280, height: 800 }});
            await page.setUserAgent('Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36');
            
            // Go to URL
            await page.goto('{}', {{ waitUntil: 'networkidle2', timeout: 30000 }});
            
            // Wait for key elements to load
            await page.waitForSelector('#content', {{ timeout: 10000 }}).catch(() => {{}});
            await page.waitForSelector('#description', {{ timeout: 5000 }}).catch(() => {{}});
            
            // Try to click "Show more" button to expand description
            try {{
                await page.click('button[aria-label="Show more"]');
                // Wait a bit for description to expand
                await page.waitForTimeout(1000);
            }} catch (e) {{
                // Ignore if button not found
            }}
            
            // Get the page HTML
            const html = await page.content();
            console.log(html);
            
            await browser.close();
        }} catch (error) {{
            console.error('Error:', error);
            process.exit(1);
        }}
    }})();
    "#, url);
    
    js_file.write_all(js_code.as_bytes())?;
    
    // Execute the JavaScript code with Node.js
    let output = Command::new("node")
        .arg(js_file.path())
        .output()
        .context("Failed to execute Node.js. Make sure it's installed and in your PATH")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        info!("JavaScript execution failed: {}. Using fallback method.", error);
        return Ok(String::new());
    }
    
    let html = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(html)
}

/// Try to get a YouTube page with various methods
fn get_youtube_page(client: &Client, url: &str) -> Result<Html, Error> {
    // First try: standard HTTP request with reqwest
    info!("Attempting regular HTTP request to YouTube...");
    let response = client.get(url)
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()?;
    
    let mut html = String::new();
    let mut document = Html::new_document();
    let mut standard_request_success = false;
    
    if response.status().is_success() {
        html = response.text()?;
        document = Html::parse_document(&html);
        
        // Check if we got what looks like a proper YouTube page
        if has_key_youtube_elements(&document) {
            info!("Successfully fetched YouTube page with regular HTTP request");
            standard_request_success = true;
        }
    }
    
    // Second try: Use JavaScript for more complex extraction
    if !standard_request_success {
        info!("Regular HTTP request didn't get complete page. Attempting JavaScript extraction...");
        let js_html = extract_with_javascript(url)?;
        
        // Only use JavaScript result if we got something back
        if !js_html.is_empty() {
            let js_document = Html::parse_document(&js_html);
            
            if has_key_youtube_elements(&js_document) {
                info!("Successfully fetched YouTube page with JavaScript");
                return Ok(js_document);
            }
        }
    }
    
    // If JavaScript extraction was empty or failed but we have content from the standard request
    if !html.is_empty() {
        info!("Using result from standard HTTP request");
        return Ok(document);
    }
    
    // If we have nothing, create a minimal valid document for further processing
    info!("Failed to get complete page, using minimal valid document");
    bail!("Could not fetch YouTube page content")
}

/// Check if the parsed HTML has key YouTube elements
fn has_key_youtube_elements(document: &Html) -> bool {
    let selectors = [
        "#content", 
        "#player", 
        "#description", 
        "title",
        "meta[property='og:title']",
    ];
    
    let mut found = 0;
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if document.select(&selector).next().is_some() {
                found += 1;
            }
        }
    }
    
    // Consider it a valid YouTube page if we found at least 3 key elements
    found >= 3
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
        "a[href*='watch?v=']",          // Any anchor with watch?v= in href
        "a[href^='/watch']",            // Any anchor with href starting with /watch
        "div#contents a[href]",         // Any link in the contents div
        "div.ytd-rich-grid-renderer a[href]", // Links in grid renderer
    ];
    
    let mut video_links = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();
    
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
                        
                        // Only add unique URLs
                        if !seen_urls.contains(&full_url) {
                            seen_urls.insert(full_url.clone());
                            video_links.push(full_url);
                        }
                    }
                }
            }
            
            // If we found a reasonable number of videos, we can stop
            if video_links.len() >= 10 {
                break;
            }
        }
    }
    
    // If we still didn't find links, look in embedded JSON data
    if video_links.is_empty() {
        info!("Trying to extract video links from embedded JSON data");
        if let Ok(script_selector) = Selector::parse("script") {
            for script in document.select(&script_selector) {
                let script_text = script.text().collect::<String>();
                
                // Look for video IDs in the script
                let video_id_regex = Regex::new(r#""videoId":\s*"([a-zA-Z0-9_-]{11})""#).unwrap();
                for cap in video_id_regex.captures_iter(&script_text) {
                    if let Some(id_match) = cap.get(1) {
                        let video_id = id_match.as_str();
                        let url = format!("https://www.youtube.com/watch?v={}", video_id);
                        
                        if !seen_urls.contains(&url) {
                            seen_urls.insert(url.clone());
                            video_links.push(url);
                        }
                    }
                }
                
                // If we found a reasonable number of videos, we can stop
                if video_links.len() >= 20 {
                    break;
                }
            }
        }
    }
    
    // Limit the number of videos to process to avoid too many requests
    let max_videos = 30;
    if video_links.len() > max_videos {
        info!("Limiting to {} videos out of {} found", max_videos, video_links.len());
        video_links = video_links.into_iter().take(max_videos).collect();
    }
    
    Ok(video_links)
}

/// Process a single YouTube video
fn process_single_video(client: &Client, url: &str) -> Result<(String, Vec<String>), Error> {
    // Fetch the video page using enhanced methods
    println!("Fetching video page: {}", url);
    let document = get_youtube_page(client, url)?;
    
    // Extract the video title
    let title = extract_video_title(&document)?;
    println!("Video title: {}", title);
    
    // Format the video title for folder name
    let formatted_title = sanitize_filename(&title);
    println!("Formatted video title: {}", formatted_title);
    
    // Extract the video description
    let description = extract_video_description(&document)?;
    println!("Description length: {} characters", description.len());
    if description.len() > 100 {
        println!("Description snippet: {}", &description[..min(100, description.len())]);
    }
    
    // Extract upload date and views
    let upload_date = extract_upload_date(&document).unwrap_or_else(|| "Unknown date".to_string());
    let views = extract_views(&document).unwrap_or_else(|| "Unknown views".to_string());
    
    // Log detailed information
    println!("Video: {} - Premiered on {} - {} - {}", formatted_title, upload_date, views, url);
    
    // Try to extract tracklist from description
    let mut tracklist = extract_tracklist_from_description(&description);
    
    // If no tracks found, try a more aggressive approach
    if tracklist.is_empty() {
        println!("No tracks found in initial description. Trying aggressive parsing...");
        
        // Try getting JavaScript-rendered version explicitly
        let js_html = match extract_with_javascript(url) {
            Ok(html) => html,
            Err(e) => {
                println!("JavaScript extraction failed: {}", e);
                // Continue with what we have
                String::new()
            }
        };
        
        if !js_html.is_empty() {
            let js_document = Html::parse_document(&js_html);
            let js_description = extract_video_description(&js_document)?;
            
            // Only try with the new description if it's different or longer
            if js_description.len() > description.len() || js_description != description {
                println!("Found different description with JavaScript, trying to extract tracklist...");
                tracklist = extract_tracklist_from_description(&js_description);
            }
        }
        
        // If still empty, try looking for tracks in the page title or other metadata
        if tracklist.is_empty() {
            println!("Still no tracks found. Checking for tracks in page metadata...");
            
            // Look for common patterns in the title that might indicate a music mix
            if title.to_lowercase().contains("mix") || 
               title.to_lowercase().contains("set") || 
               title.to_lowercase().contains("playlist") {
                
                // Try the meta tags for song information
                if let Ok(meta_selector) = Selector::parse("meta[property='og:audio:title'], meta[property='og:audio:artist']") {
                    for element in document.select(&meta_selector) {
                        if let Some(content) = element.value().attr("content") {
                            if content.contains("-") {
                                // This might be a track in "Artist - Title" format
                                let clean_track = clean_track_text(content);
                                if !clean_track.is_empty() {
                                    println!("Found track in meta tag: {}", clean_track);
                                    tracklist.push(clean_track);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Create a more detailed log for debugging
    let mut detailed_log = format!("Video 1: {} - Premiered on {} - {} - {}\n{} tracks found\n\n", 
                                formatted_title, upload_date, views, url, tracklist.len());
    
    for (i, track) in tracklist.iter().enumerate() {
        detailed_log.push_str(&format!("{}. {}\n", i + 1, track));
    }
    
    println!("{}", detailed_log);
    
    Ok((formatted_title, tracklist))
}

fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

/// Extract upload date from a YouTube video page
fn extract_upload_date(document: &Html) -> Option<String> {
    let selectors = [
        "meta[itemprop='uploadDate']",
        "meta[property='uploadDate']",
        "#info-strings yt-formatted-string",
        "#info-text .style-scope"
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                // Check for content attribute (meta tags)
                if let Some(content) = element.value().attr("content") {
                    if !content.is_empty() {
                        return Some(content.to_string());
                    }
                }
                
                // Check for text content
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() && (text.contains("Premiered") || text.contains("Uploaded") || text.contains("Streamed")) {
                    return Some(text);
                }
            }
        }
    }
    
    None
}

/// Extract view count from a YouTube video page
fn extract_views(document: &Html) -> Option<String> {
    let selectors = [
        "meta[itemprop='interactionCount']",
        "#count .view-count",
        "#info-text .style-scope",
        ".view-count"
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                // Check for content attribute (meta tags)
                if let Some(content) = element.value().attr("content") {
                    if !content.is_empty() {
                        return Some(format!("{} views", content));
                    }
                }
                
                // Check for text content
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() && (text.contains("view") || text.contains("views")) {
                    return Some(text);
                }
            }
        }
    }
    
    None
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
    // First try to find a full description by checking JSON-LD and other structured data
    if let Some(structured_desc) = extract_from_structured_data(document) {
        if structured_desc.contains("Tracklist") || 
           structured_desc.contains("track list") || 
           structured_desc.to_lowercase().contains("tracklist") || 
           structured_desc.contains("00:00") {
            info!("Found description in structured data that may contain tracklist");
            return Ok(structured_desc);
        }
    }
    
    // Try the meta description first as a quick check
    let meta_description = if let Ok(meta_selector) = Selector::parse("meta[property='og:description']") {
        document.select(&meta_selector)
            .next()
            .and_then(|e| e.value().attr("content"))
            .map(|s| s.to_string())
    } else {
        None
    };
    
    // If we have a meta description that might contain tracklist info, use it
    if let Some(desc) = &meta_description {
        if desc.contains("Tracklist") || 
           desc.contains("track list") || 
           desc.to_lowercase().contains("tracklist") || 
           desc.contains("00:00") {
            info!("Found potential tracklist in meta description");
            return Ok(desc.clone());
        }
    }
    
    // Try to find the description using various selectors that match YouTube's DOM structure
    // These are listed in order of preference (most reliable first)
    let selectors = [
        // Modern YouTube selectors (2023-2025)
        "#description-inner",
        "#description-text",
        "#description .content",
        "#description-inline-expander",
        "ytd-watch-metadata #description",
        "ytd-watch-metadata #description yt-formatted-string",
        "ytd-watch-metadata #description-input-container",
        "ytd-expander #content yt-formatted-string",
        "ytd-expander[collapsed] #content",
        "ytd-expander[expanded] #content",
        
        // Older YouTube selectors
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
        
        // Generic selectors
        "#description",
        "#info-contents",
        "#meta",
    ];
    
    // Try each selector for the description
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let elements: Vec<_> = document.select(&selector).collect();
            
            // If we found elements, try to get their text content
            if !elements.is_empty() {
                let mut full_text = String::new();
                
                for element in elements {
                    // Get all text nodes, preserving line breaks
                    let text = element.text().collect::<Vec<_>>().join("\n");
                    if !text.trim().is_empty() {
                        full_text.push_str(&text);
                        full_text.push('\n');
                    }
                }
                
                // Clean up the text
                let trimmed = full_text.trim().to_string();
                if !trimmed.is_empty() {
                    info!("Found description with selector: {}", selector_str);
                    return Ok(trimmed);
                }
            }
        }
    }
    
    // If we still haven't found anything, try the meta description as a fallback
    if let Some(desc) = meta_description {
        if !desc.is_empty() {
            info!("Using meta description as fallback");
            return Ok(desc);
        }
    }
    
    // Try collecting all text in #description or similar elements
    if let Ok(desc_selector) = Selector::parse("#description, #info-text, .description") {
        let mut full_text = String::new();
        for element in document.select(&desc_selector) {
            full_text.push_str(&element.text().collect::<String>());
            full_text.push('\n');
        }
        
        if !full_text.is_empty() {
            info!("Collected description text from multiple elements");
            return Ok(full_text);
        }
    }
    
    // If all else fails, return a placeholder
    warn!("Could not find video description");
    Ok("Description not found".to_string())
}

/// Attempt to extract description from structured data in the page
fn extract_from_structured_data(document: &Html) -> Option<String> {
    // Try to find JSON-LD script tags
    if let Ok(script_selector) = Selector::parse("script[type='application/ld+json']") {
        for script in document.select(&script_selector) {
            let script_text = script.text().collect::<String>();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&script_text) {
                // Look for the description field
                if let Some(desc) = json.get("description") {
                    if let Some(desc_str) = desc.as_str() {
                        return Some(desc_str.to_string());
                    }
                }
            }
        }
    }
    
    // Try to extract from ytInitialData
    if let Ok(script_selector) = Selector::parse("script") {
        for script in document.select(&script_selector) {
            let script_text = script.text().collect::<String>();
            
            // Find ytInitialData
            if script_text.contains("var ytInitialData = ") {
                if let Some(start) = script_text.find("var ytInitialData = ") {
                    let data_start = start + "var ytInitialData = ".len();
                    
                    // Find the end of the JSON object (usually ends with }; or })
                    if let Some(end_pos) = script_text[data_start..].find("};") {
                        let json_str = &script_text[data_start..data_start + end_pos + 1];
                        
                        // Try to parse the JSON
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                            // Try to navigate to the description
                            // This is a simplification - the actual path may vary
                            if let Some(video_primary_info) = data
                                .get("contents")
                                .and_then(|c| c.get("twoColumnWatchNextResults"))
                                .and_then(|w| w.get("results"))
                                .and_then(|r| r.get("results"))
                                .and_then(|r| r.get("contents")) {
                                
                                // Iterate through the contents to find the description
                                if let Some(contents) = video_primary_info.as_array() {
                                    for content in contents {
                                        if let Some(description) = content
                                            .get("videoSecondaryInfoRenderer")
                                            .and_then(|v| v.get("description"))
                                            .and_then(|d| d.get("runs")) {
                                            
                                            // Combine all the text runs
                                            if let Some(runs) = description.as_array() {
                                                let mut desc_text = String::new();
                                                for run in runs {
                                                    if let Some(text) = run.get("text").and_then(|t| t.as_str()) {
                                                        desc_text.push_str(text);
                                                    }
                                                }
                                                
                                                if !desc_text.is_empty() {
                                                    return Some(desc_text);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Try to find literal tracklist in script content
    if let Ok(script_selector) = Selector::parse("script") {
        for script in document.select(&script_selector) {
            let script_text = script.text().collect::<String>();
            
            // Check for tracklist indicators
            if script_text.contains("Tracklist") && script_text.contains("00:00") {
                let mut tracklist_text = String::new();
                let mut in_tracklist = false;
                
                for line in script_text.lines() {
                    if line.contains("Tracklist") || line.contains("Track list") || line.contains("TRACKLIST") {
                        in_tracklist = true;
                        tracklist_text.push_str(line);
                        tracklist_text.push('\n');
                    } else if in_tracklist && line.contains(":") && 
                              (line.contains("-") || line.contains(".") || line.contains(")")) {
                        tracklist_text.push_str(line);
                        tracklist_text.push('\n');
                    } else if in_tracklist && tracklist_text.len() > 100 && line.trim().is_empty() {
                        // End of tracklist section
                        break;
                    }
                }
                
                if tracklist_text.len() > 50 {
                    return Some(tracklist_text);
                }
            }
        }
    }
    
    None
}

/// Extract tracklist from video description
fn extract_tracklist_from_description(description: &str) -> Vec<String> {
    // For shorter descriptions, use the entire thing
    // For longer descriptions, try to find the tracklist section
    let tracklist_section = if description.len() < 500 {
        description
    } else if let Some(section) = find_tracklist_section(description) {
        section
    } else {
        description
    };
    
    info!("Analyzing text section. First 200 chars: {}", 
          &tracklist_section[..min(200, tracklist_section.len())]);
    
    // Extract track entries
    let mut tracks = Vec::new();
    let mut seen_tracks = std::collections::HashSet::new();
    
    // First try the most structured tracklist pattern (numbered with timestamps)
    let structured_tracks = extract_structured_tracklist(tracklist_section);
    if !structured_tracks.is_empty() {
        info!("Found {} tracks using structured pattern", structured_tracks.len());
        return structured_tracks;
    }
    
    // Now try more general timestamp patterns
    // First try looking for timestamped lines
    let timestamp_lines: Vec<_> = tracklist_section
        .lines()
        .filter(|line| line.contains(":"))
        .filter(|line| {
            let has_digits = line.chars().any(|c| c.is_digit(10));
            let has_timestamp = Regex::new(r"\d+:\d+").unwrap().is_match(line);
            has_digits && has_timestamp
        })
        .collect();
    
    info!("Found {} lines with timestamps", timestamp_lines.len());
    for (i, line) in timestamp_lines.iter().enumerate().take(5) {
        debug!("Timestamp line {}: {}", i+1, line);
    }
    
    // Special handling for YouTube descriptions - look for known patterns
    for line in tracklist_section.lines() {
        // First look for numbered entries with timestamps and artist/title
        // Pattern like: "1. 00:00 Artist - Title" or "1) 00:00 Artist - Title"
        if line.contains(":") && 
           (line.contains(" - ") || line.contains("- ") || line.contains(" -")) && 
           (line.contains(".") || line.contains(")") || Regex::new(r"^\s*\d+\s").unwrap().is_match(line)) && 
           Regex::new(r"\d{1,2}:\d{2}(?::\d{2})?").unwrap().is_match(line) {
            
            // Try to extract the artist/title part
            if let Some(dash_pos) = line.find(" - ").or_else(|| line.find("- ")).or_else(|| line.find(" -")) {
                // Find timestamp pattern with support for HH:MM:SS format
                if let Some(timestamp_end) = Regex::new(r"\d{1,2}:\d{2}(?::\d{2})?").unwrap()
                    .find_iter(line)
                    .filter(|m| m.end() < dash_pos)
                    .last()
                    .map(|m| m.end()) {
                    
                    // Extract everything after timestamp
                    let track_text = line[timestamp_end..].trim();
                    
                    // Clean it up
                    let clean_track = clean_track_text(track_text);
                    if !clean_track.is_empty() && !seen_tracks.contains(&clean_track) {
                        debug!("Found track with timestamp-dash pattern: {}", clean_track);
                        seen_tracks.insert(clean_track.clone());
                        tracks.push(clean_track);
                    }
                }
            }
        }
        // Also look for just timestamp followed by artist - title
        else if line.contains(":") && 
                (line.contains(" - ") || line.contains("- ") || line.contains(" -")) && 
                Regex::new(r"\d{1,2}:\d{2}(?::\d{2})?\s+\S+").unwrap().is_match(line) {
            
            // Find timestamp pattern with support for HH:MM:SS format
            if let Some(timestamp_match) = Regex::new(r"\d{1,2}:\d{2}(?::\d{2})?").unwrap()
                .find_iter(line)
                .last() {
                
                // Extract everything after the timestamp
                let track_text = line[timestamp_match.end()..].trim();
                
                // Clean it up
                let clean_track = clean_track_text(track_text);
                if !clean_track.is_empty() && !seen_tracks.contains(&clean_track) {
                    debug!("Found track with timestamp pattern: {}", clean_track);
                    seen_tracks.insert(clean_track.clone());
                    tracks.push(clean_track);
                }
            }
        }
    }
    
    // If we still don't have enough tracks, try more generic patterns
    if tracks.len() < 5 {
        info!("First attempt found too few tracks ({}), trying alternate patterns...", tracks.len());
        
        // Try looking for timestamps without artist-title structure
        if timestamp_lines.len() >= 5 {
            info!("Found {} timestamp lines, attempting to extract artist/title", timestamp_lines.len());
            
            // Look for Artist - Title patterns in each line with timestamp
            for line in &timestamp_lines {
                if let Some(timestamp_match) = Regex::new(r"\d{1,2}:\d{2}(?::\d{2})?").unwrap().find(line) {
                    let after_timestamp = &line[timestamp_match.end()..].trim();
                    
                    // Try to find Artist - Title pattern
                    let dash_pattern = Regex::new(r"([^-]+)-([^-]+)").unwrap();
                    if let Some(caps) = dash_pattern.captures(after_timestamp) {
                        if caps.len() >= 3 {
                            let artist = caps.get(1).unwrap().as_str().trim();
                            let title = caps.get(2).unwrap().as_str().trim();
                            
                            if !artist.is_empty() && !title.is_empty() {
                                let track = format!("{} - {}", artist, title);
                                let clean_track = clean_track_text(&track);
                                
                                if !clean_track.is_empty() && !seen_tracks.contains(&clean_track) {
                                    debug!("Extracted track from timestamp line: {}", clean_track);
                                    seen_tracks.insert(clean_track.clone());
                                    tracks.push(clean_track);
                                }
                            }
                        }
                    } else if !after_timestamp.is_empty() {
                        // If there's no dash pattern but there is content, use it as is
                        let clean_text = clean_track_text(after_timestamp);
                        if !clean_text.is_empty() && !seen_tracks.contains(&clean_text) {
                            debug!("Extracted text after timestamp: {}", clean_text);
                            seen_tracks.insert(clean_text.clone());
                            tracks.push(clean_text);
                        }
                    }
                }
            }
        }
        
        // Try extracting any artist-title pattern in the text
        let artist_title_pattern = Regex::new(r"([A-Za-z0-9&\s,\.]+)\s+-\s+([A-Za-z0-9&\s,\.']+)").unwrap();
        
        for cap in artist_title_pattern.captures_iter(tracklist_section) {
            if cap.len() >= 3 {
                let artist = cap.get(1).unwrap().as_str().trim();
                let title = cap.get(2).unwrap().as_str().trim();
                
                // Skip very short artist/title combos
                if artist.len() > 2 && title.len() > 2 {
                    let track = format!("{} - {}", artist, title);
                    
                    // Clean up
                    let clean_track = clean_track_text(&track);
                    if !seen_tracks.contains(&clean_track) {
                        debug!("Found track with artist-title pattern: {}", clean_track);
                        seen_tracks.insert(clean_track.clone());
                        tracks.push(clean_track);
                    }
                }
            }
        }
    }
    
    // If we still don't have enough tracks but we have lines with timestamps,
    // make a desperate attempt to parse them
    if tracks.len() < 5 && timestamp_lines.len() >= 5 {
        info!("Still found few tracks, making another attempt with timestamp lines");
        
        // Just use the lines with timestamps directly, after minimal cleaning
        for line in &timestamp_lines {
            let clean_line = clean_basic_timestamp(line);
            if !clean_line.is_empty() && !seen_tracks.contains(&clean_line) {
                debug!("Using timestamp line directly: {}", clean_line);
                seen_tracks.insert(clean_line.clone());
                tracks.push(clean_line);
            }
        }
    }
    
    // Print the number of tracks found
    info!("{} tracks found", tracks.len());
    tracks
}

/// Extract a well-structured tracklist with numbered entries
fn extract_structured_tracklist(text: &str) -> Vec<String> {
    let mut tracks = Vec::new();
    let mut seen_tracks = std::collections::HashSet::new();
    
    // Look for the "Tracklist:" section
    let tracklist_section = if let Some(idx) = text.to_lowercase().find("tracklist:") {
        &text[idx..]
    } else if let Some(idx) = text.to_lowercase().find("tracklist") {
        &text[idx..]
    } else {
        text
    };
    
    // Pattern for "1. 00:00 Artist - Title"
    if let Ok(numbered_pattern) = Regex::new(r"(?m)^\s*(\d+)[\.\)]\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+(.+)$") {
        // If the pattern matches at least 3 times, we likely have a structured tracklist
        let matches: Vec<_> = numbered_pattern.captures_iter(tracklist_section).collect();
        
        if matches.len() >= 3 {
            for cap in matches {
                if cap.len() >= 4 {
                    // Extract the text after the timestamp
                    let track_text = cap.get(3).unwrap().as_str().trim();
                    
                    // Clean it up
                    let clean_track = clean_track_text(track_text);
                    if !clean_track.is_empty() && !seen_tracks.contains(&clean_track) {
                        seen_tracks.insert(clean_track.clone());
                        tracks.push(clean_track);
                    }
                }
            }
        }
    }
    
    tracks
}

/// Clean a timestamp line by removing the timestamp and basic noise
fn clean_basic_timestamp(line: &str) -> String {
    // Remove timestamp and leading track number if present
    let timestamp_pattern = Regex::new(r"^\s*(?:\d+[\.\)]\s*)?\d{1,2}:\d{2}(?::\d{2})?\s*").unwrap();
    let without_timestamp = timestamp_pattern.replace(line, "").to_string();
    
    // Try to clean it up a bit more
    clean_track_text(&without_timestamp)
}

/// Find the tracklist section in a video description
fn find_tracklist_section(description: &str) -> Option<&str> {
    // Look for common tracklist section indicators, case insensitive
    let tracklist_markers = [
        "Tracklist:", "Track list:", "Tracks:", "Songs:", "Music:", "Playlist:", 
        "Track List", "TRACKLIST", "00:00", "0:00", "1. ", "01. ",
        "1) ", "01) ", "1 - ", "01 - ",
    ];
    
    // Try to find explicit tracklist markers first
    for marker in &tracklist_markers {
        let marker_lower = marker.to_lowercase();
        if let Some(pos) = description.to_lowercase().find(&marker_lower) {
            // Check if this is a line starting with the marker to avoid false positives
            let line_start = description[..pos].rfind('\n').map_or(0, |p| p + 1);
            let prefix = &description[line_start..pos];
            if prefix.trim().is_empty() || marker.starts_with(|c: char| c.is_numeric()) {
                info!("Found tracklist section with marker: {}", marker);
                return Some(&description[pos..]);
            }
        }
    }
    
    // Check for numeric patterns that might indicate a tracklist
    // Look for lines that start with numbers followed by timestamps
    let timestamp_pattern = regex::Regex::new(r"(?m)^\s*\d+[.)\s]*\d{1,2}:\d{2}").ok()?;
    if let Some(mat) = timestamp_pattern.find(description) {
        info!("Found tracklist section with timestamp pattern at position {}", mat.start());
        return Some(&description[mat.start()..]);
    }
    
    // If we've tried everything and found nothing specific, return the whole description
    // but log a warning
    info!("No specific tracklist section found, using entire description");
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
    let timestamp_re = Regex::new(r"^\s*(?:\d+[\.\):]?\s*)?\d{1,2}:\d{2}(?::\d{2})?\s+").unwrap();
    let without_timestamp = timestamp_re.replace_all(track, "").to_string();
    
    // Also remove timestamps that might be embedded in the string
    let embedded_timestamp_re = Regex::new(r"\s+\d{1,2}:\d{2}(?::\d{2})?\s+").unwrap();
    let clean_timestamps = embedded_timestamp_re.replace_all(&without_timestamp, " ").to_string();
    
    // Try to handle common HTML entities
    let decoded = clean_timestamps
        .replace("u0026", "&")
        .replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&quot;", "\"");
    
    // Remove social media handles and other noise
    let without_social = Regex::new(r"@\w+|\(@\w+\)|ÔÇ¬ÔÇ¼X?|X$|X\s*$").unwrap()
        .replace_all(&decoded, "")
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
    
    // Try to extract artist and title if the track contains " - "
    if fixed_encoding.contains(" - ") {
        let parts: Vec<&str> = fixed_encoding.split(" - ").collect();
        if parts.len() >= 2 {
            let artist = parts[0].trim();
            let title = parts[1].trim();
            
            // Check for timestamps in artist name and remove them
            let artist_clean = timestamp_re.replace_all(artist, "").to_string();
            
            if !artist_clean.is_empty() && !title.is_empty() {
                return format!("{} - {}{}", artist_clean, title, remix_info);
            }
        }
    }
    
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
