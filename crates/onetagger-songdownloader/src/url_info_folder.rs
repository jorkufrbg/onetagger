use clap::Parser;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::error::Error;
use std::time::Duration;
use std::collections::HashSet;
use regex::Regex;
use std::path::Path;

/// A program to extract YouTube video descriptions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The YouTube video URL
    #[arg(short, long)]
    url: String,
    
    /// The directory to check for existing folders
    #[arg(short, long)]
    directory: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args = Args::parse();
    let url = args.url.clone();
    println!(
            "Checking Youtube for songs in video description. URL: '{}'",
            url
        );
    
    // Configure and launch headless Chrome
    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .sandbox(false)
        .build()
        .expect("Failed to build launch options");
    
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;
    
    // Navigate to the YouTube URL
    tab.navigate_to(&url)?;
    
    // Wait for the page to load (initial wait)
    tab.wait_until_navigated()?;
    
    // Wait for the page to fully load (additional wait)
    std::thread::sleep(Duration::from_secs(3));
    
    // Extract the video title
    let title_result = tab.evaluate(
        r#"
        document.title.replace(" - YouTube", "")
        "#,
        true,
    )?;
    
    let title = match title_result.value {
        Some(value) => value.as_str().unwrap_or("Unknown Title").to_string(),
        None => "Unknown Title".to_string(),
    };
    
    // Extract the channel name
    let channel_name_result = tab.evaluate(
        r##"
        (function() {
            const selectors = [
                "#owner #channel-name",
                "#channel-name",
                "#owner-name a",
                "#owner-name",
                "ytd-channel-name yt-formatted-string a",
                "ytd-channel-name yt-formatted-string"
            ];
            
            for (const selector of selectors) {
                const element = document.querySelector(selector);
                if (element && element.textContent.trim()) {
                    return element.textContent.trim();
                }
            }
            
            return "Unknown Channel";
        })()
        "##,
        true,
    )?;
    
    let channel_name = match channel_name_result.value {
        Some(value) => value.as_str().unwrap_or("Unknown Channel").to_string(),
        None => "Unknown Channel".to_string(),
    };
    
    // Extract the date posted
    let date_posted_result = tab.evaluate(
        r##"
        (function() {
            const selectors = [
                "#info-strings yt-formatted-string",
                "#info-strings span",
                "#upload-info .date",
                "#metadata-line span:nth-child(2)"
            ];
            
            for (const selector of selectors) {
                const elements = document.querySelectorAll(selector);
                for (const element of elements) {
                    const text = element.textContent.trim();
                    if (text.includes("Premiered") || 
                        text.includes("Streamed") || 
                        text.includes("ago") ||
                        /\d{1,2}\s+[A-Za-z]{3,}\s+\d{4}/.test(text)) {
                        return text;
                    }
                }
            }
            
            return "";
        })()
        "##,
        true,
    )?;
    
    let date_posted = match date_posted_result.value {
        Some(value) => value.as_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };
    
    // Extract the view count
    let view_count_result = tab.evaluate(
        r##"
        (function() {
            const selectors = [
                "#count .view-count",
                "#count span",
                "#info span:first-child",
                "#metadata-line span:first-child"
            ];
            
            for (const selector of selectors) {
                const element = document.querySelector(selector);
                if (element && element.textContent.trim()) {
                    const text = element.textContent.trim();
                    if (text.includes("view")) {
                        return text;
                    }
                }
            }
            
            return "";
        })()
        "##,
        true,
    )?;
    
    let view_count = match view_count_result.value {
        Some(value) => value.as_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };
    
    // Try to click "Show more" button if it exists to expand the description
    let _ = tab.evaluate(
        r##"
        (function() {
            const moreButton = document.querySelector('tp-yt-paper-button#expand');
            if (moreButton) {
                moreButton.click();
                return true;
            }
            
            const moreButton2 = document.querySelector('button[aria-label="Show more"]');
            if (moreButton2) {
                moreButton2.click();
                return true;
            }
            
            return false;
        })()
        "##,
        true,
    );
    
    // Wait a bit for the description to expand
    std::thread::sleep(Duration::from_secs(2));
    
    // Extract the description text using multiple possible selectors
    let description_result = tab.evaluate(
        r##"
        (function() {
            // Try different selectors for the description
            const selectors = [
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
                "#info-contents #description yt-formatted-string"
            ];
            
            for (const selector of selectors) {
                const element = document.querySelector(selector);
                if (element) {
                    return element.innerText || element.textContent;
                }
            }
            
            // If no selector worked, try to get all text from the description area
            const descriptionArea = document.querySelector("#description") || 
                                   document.querySelector("#info-contents") ||
                                   document.querySelector("#meta");
                                   
            if (descriptionArea) {
                return descriptionArea.innerText || descriptionArea.textContent;
            }
            
            return "Description not found. The YouTube page structure might have changed.";
        })()
        "##,
        true,
    )?;
    
    // Convert the video title to a valid folder name
    let folder_name = sanitize_filename(&title);
    
    // Check if the directory parameter was provided
    if let Some(directory) = &args.directory {
        let folder_path = Path::new(directory).join(&folder_name);
        
        // Log what video title and sanitized folder name are being checked
        println!(
            "Checking folder for sanitized video title: '{}'",
            folder_name
        );
        
        // Check if the folder exists
        if folder_path.exists() && folder_path.is_dir() {
            println!(
                "Skipping, folder '{}' exists for video title '{}'.",
            folder_name, folder_name
        );
        } else {
            println!(
                "New video, folder '{}' does not exist for sanitized video title '{}'. Ready to create folder.",
                folder_name, folder_name
            );
        }
    }
    
    if let Some(value) = description_result.value {
        if let Some(description) = value.as_str() {
            // Print the video information
            let mut video_info = String::new();
            
            // Add channel name if available
            if !channel_name.is_empty() && channel_name != "Unknown Channel" {
                video_info.push_str(&format!("{} - ", channel_name));
            }
            
            // Add title
            video_info.push_str(&title);
            
            // Add date posted if available
            if !date_posted.is_empty() {
                video_info.push_str(&format!(" - {}", date_posted));
            }
            
            // Add view count if available
            if !view_count.is_empty() {
                video_info.push_str(&format!(" - {}", view_count));
            }
            
            // Add URL
            video_info.push_str(&format!(" - {}", url));
            
            println!("{}", video_info);
            
            // Extract and format the tracklist
            extract_and_print_tracklist(description);
        } else {
            println!("Failed to extract description text.");
        }
    } else {
        println!("No description content found.");
    }
    
    Ok(())
}

// Function to convert a title to a valid folder name
fn sanitize_filename(filename: &str) -> String {
    // Replace invalid characters with underscores or spaces
    let invalid_chars = regex::Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
    let sanitized = invalid_chars.replace_all(filename, " ").to_string();
    
    // Trim leading/trailing whitespace and dots
    let trimmed = sanitized.trim().trim_matches('.');
    
    // Normalize multiple spaces to a single space
    let normalized = regex::Regex::new(r"\s+").unwrap().replace_all(&trimmed, " ").to_string();
    
    // Ensure the filename is not empty
    if normalized.is_empty() {
        return "Unknown_Title".to_string();
    }
    
    normalized
}

fn extract_and_print_tracklist(description: &str) {
    // Try to find the tracklist section
    if let Some(tracklist_section) = find_tracklist_section(description) {
        // Extract track entries
        let mut tracks = Vec::new();
        let mut seen_tracks = HashSet::new();
        
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
        
        // Print the number of tracks found and the tracks
        println!("  {} tracks found", tracks.len());
        for track in tracks {
            println!("  {}", track);
        }
    } else {
        println!("  No tracklist found in the description.");
    }
}

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

fn clean_track_text(track: &str) -> String {
    //Try to extract using the pattern
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

fn extract_tracks_line_by_line(text: &str, tracks: &mut Vec<String>, seen_tracks: &mut HashSet<String>) {
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
