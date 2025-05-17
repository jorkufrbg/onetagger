use anyhow::{Error, bail, Context};
use log::{info, warn};
use regex::Regex;
use crate::UrlInfo;
// Commented imports to be uncommented once rspotify is properly linked
// use onetagger_platforms::spotify::{Spotify, Settings};
// use rspotify::prelude::*;
// use rspotify::AuthCodeSpotify;
// use rspotify::model::{PlaylistId, AlbumId, TrackId, ArtistId};

/// Process a Spotify URL
pub fn process_spotify(url: &str, confidence: f32) -> Result<UrlInfo, Error> {
    // For now, just return a skeleton implementation since we're having build issues
    // Once the build is fixed, we can implement the full version
    
    // Extract content type from URL
    let content_type = if url.contains("/track/") {
        "track"
    } else if url.contains("/album/") {
        "album"
    } else if url.contains("/playlist/") {
        "playlist"
    } else if url.contains("/artist/") {
        "artist"
    } else {
        "unknown"
    };
    
    // Create a placeholder title
    let title = format!("Spotify {} from {}", content_type, url);
    
    // Create a simple UrlInfo with just the basics
    let url_info = UrlInfo::new(
        "spotify",
        content_type,
        &title,
        None
    )
    .with_url(url.to_string());
    
    Ok(url_info)
}

/// Create an authenticated Spotify client - Commented until dependencies are properly linked
/* 
fn create_spotify_client() -> Result<AuthCodeSpotify, Error> {
    // Try to get settings
    let settings = Settings::get();
    
    // Check if Spotify config exists
    if settings.spotify.client_id.is_empty() || settings.spotify.client_secret.is_empty() {
        bail!("Spotify client ID and secret are not configured. Please run the main OneTagger application and configure Spotify.");
    }
    
    // Try to use cached token first
    if let Some(spotify) = Spotify::try_cached_token(
        &settings.spotify.client_id,
        &settings.spotify.client_secret
    ) {
        info!("Using cached Spotify authentication token");
        return Ok(spotify.spotify);
    }
    
    // If no cached token, notify user to authenticate
    warn!("Spotify authentication required. Please run the main OneTagger application to authenticate with Spotify.");
    bail!("Spotify authentication required. Run the main OneTagger application first.")
}
*/

/// Extract content type and ID from Spotify URL - Commented until dependencies are properly linked
/* 
fn extract_spotify_id(url: &str) -> Result<(String, String), Error> {
    // Handle URLs like: https://open.spotify.com/track/4iV5W9uYEdYUVa79Axb7Rh
    // or https://open.spotify.com/album/1DFixLWuPkv3KT3TnV35m3
    
    let url_regex = Regex::new(r"spotify\.com/([a-z]+)/([a-zA-Z0-9]+)").unwrap();
    if let Some(caps) = url_regex.captures(url) {
        let content_type = caps.get(1).unwrap().as_str().to_string();
        let id = caps.get(2).unwrap().as_str().to_string();
        return Ok((content_type, id));
    }
    
    // Handle Spotify URIs: spotify:track:4iV5W9uYEdYUVa79Axb7Rh
    let uri_regex = Regex::new(r"spotify:([a-z]+):([a-zA-Z0-9]+)").unwrap();
    if let Some(caps) = uri_regex.captures(url) {
        let content_type = caps.get(1).unwrap().as_str().to_string();
        let id = caps.get(2).unwrap().as_str().to_string();
        return Ok((content_type, id));
    }
    
    bail!("Invalid Spotify URL format: {}", url)
}
*/

/* Spotify implementation functions are commented out until dependencies are properly linked

/// Process a Spotify track
fn process_spotify_track(client: &AuthCodeSpotify, id: &str, url: &str) -> Result<UrlInfo, Error> {
    // Get track details
    info!("Fetching Spotify track: {}", id);
    let track_id = TrackId::from_id(id)?;
    let track = client.track(track_id, None)
        .context("Failed to fetch track from Spotify")?;
    
    // Create track title in the format "Artist - Title"
    let artists = track.artists.iter()
        .map(|a| a.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    
    let title = format!("{} - {}", artists, track.name);
    
    // Create videos vector with a single entry (empty tracklist for individual tracks)
    let track_url = format!("https://open.spotify.com/track/{}", id);
    let videos = vec![(title.clone(), track_url, Vec::<String>::new())];
    
    // Create UrlInfo
    let url_info = UrlInfo::new(
        "spotify",
        "track",
        &title,
        None
    )
    .with_videos(videos)
    .with_url(url.to_string());
    
    info!("Successfully processed Spotify track: {}", title);
    Ok(url_info)
}

/// Process a Spotify album
fn process_spotify_album(client: &AuthCodeSpotify, id: &str, url: &str) -> Result<UrlInfo, Error> {
    // Get album details
    info!("Fetching Spotify album: {}", id);
    let album_id = AlbumId::from_id(id)?;
    let album = client.album(album_id.clone(), None)
        .context("Failed to fetch album from Spotify")?;
    
    // Get tracks in album
    let mut tracks = Vec::new();
    let mut offset = 0;
    
    info!("Fetching album tracks");
    loop {
        let page = client.album_tracks(album_id.clone(), None, Some(50), Some(offset))
            .context("Failed to fetch album tracks")?;
        
        if page.items.is_empty() {
            break;
        }
        
        tracks.extend(page.items);
        offset += page.items.len() as u32;
        
        if page.next.is_none() {
            break;
        }
        
        // Add delay to avoid rate limiting
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    
    info!("Found {} tracks in album", tracks.len());
    
    // Create videos vector with all tracks
    let mut videos = Vec::new();
    for track in tracks {
        // Create artist string
        let artists = track.artists.iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        
        let track_title = format!("{} - {}", artists, track.name);
        let track_url = format!("https://open.spotify.com/track/{}", track.id.unwrap().id());
        
        // For album tracks, we use empty tracklist since they're individual tracks
        videos.push((track_title, track_url, Vec::<String>::new()));
    }
    
    // Create album artist string for the title
    let album_artists = album.artists.iter()
        .map(|a| a.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    
    let album_title = format!("{} - {}", album_artists, album.name);
    
    // Create UrlInfo
    let url_info = UrlInfo::new(
        "spotify",
        "album",
        &album_title,
        None
    )
    .with_videos(videos)
    .with_url(url.to_string());
    
    info!("Successfully processed Spotify album: {}", album_title);
    Ok(url_info)
}

/// Process a Spotify playlist
fn process_spotify_playlist(client: &AuthCodeSpotify, id: &str, url: &str) -> Result<UrlInfo, Error> {
    // Get playlist details
    info!("Fetching Spotify playlist: {}", id);
    let playlist_id = PlaylistId::from_id(id)?;
    let playlist = client.playlist(playlist_id.clone(), None, None)
        .context("Failed to fetch playlist from Spotify")?;
    
    // Get tracks in playlist
    let mut tracks = Vec::new();
    let mut offset = 0;
    
    info!("Fetching playlist items");
    loop {
        let page = client.playlist_items(playlist_id.clone(), None, None, Some(50), Some(offset))
            .context("Failed to fetch playlist items")?;
        
        if page.items.is_empty() {
            break;
        }
        
        tracks.extend(page.items);
        offset += page.items.len() as u32;
        
        if page.next.is_none() {
            break;
        }
        
        // Add delay to avoid rate limiting
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    
    info!("Found {} items in playlist", tracks.len());
    
    // Create videos vector with all tracks
    let mut videos = Vec::new();
    for item in tracks {
        if let Some(track) = item.track {
            if let Some(full_track) = track.as_full_track() {
                // Create artist string
                let artists = full_track.artists.iter()
                    .map(|a| a.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                let track_title = format!("{} - {}", artists, full_track.name);
                let track_url = format!("https://open.spotify.com/track/{}", full_track.id.unwrap().id());
                
                // For playlist tracks, we use empty tracklist since they're individual tracks
                videos.push((track_title, track_url, Vec::<String>::new()));
            }
        }
    }
    
    // Create owner info if available
    let owner_info = if let Some(owner) = playlist.owner {
        format!(" (by {})", owner.display_name.unwrap_or_else(|| owner.id.to_string()))
    } else {
        String::new()
    };
    
    let playlist_title = format!("Playlist: {}{}", playlist.name, owner_info);
    
    // Create UrlInfo
    let url_info = UrlInfo::new(
        "spotify",
        "playlist",
        &playlist_title,
        None
    )
    .with_videos(videos)
    .with_url(url.to_string());
    
    info!("Successfully processed Spotify playlist: {}", playlist_title);
    Ok(url_info)
}

/// Process a Spotify artist
fn process_spotify_artist(client: &AuthCodeSpotify, id: &str, url: &str) -> Result<UrlInfo, Error> {
    // Get artist details
    info!("Fetching Spotify artist: {}", id);
    let artist_id = ArtistId::from_id(id)?;
    let artist = client.artist(artist_id.clone())
        .context("Failed to fetch artist from Spotify")?;
    
    // Get top tracks
    info!("Fetching artist's top tracks");
    let top_tracks = client.artist_top_tracks(artist_id, None)
        .context("Failed to fetch artist's top tracks")?;
    
    info!("Found {} top tracks for artist", top_tracks.len());
    
    // Create videos vector with top tracks
    let mut videos = Vec::new();
    for track in top_tracks {
        // Create artist string
        let artists = track.artists.iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        
        let track_title = format!("{} - {}", artists, track.name);
        let track_url = format!("https://open.spotify.com/track/{}", track.id.unwrap().id());
        
        // For artist top tracks, we use empty tracklist since they're individual tracks
        videos.push((track_title, track_url, Vec::<String>::new()));
    }
    
    let artist_title = format!("Artist: {}", artist.name);
    
    // Create UrlInfo
    let url_info = UrlInfo::new(
        "spotify",
        "artist",
        &artist_title,
        None
    )
    .with_videos(videos)
    .with_url(url.to_string());
    
    info!("Successfully processed Spotify artist: {}", artist_title);
    Ok(url_info)
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_spotify_id() {
        // Test regular URLs
        let (content_type, id) = extract_spotify_id("https://open.spotify.com/track/4iV5W9uYEdYUVa79Axb7Rh").unwrap();
        assert_eq!(content_type, "track");
        assert_eq!(id, "4iV5W9uYEdYUVa79Axb7Rh");
        
        let (content_type, id) = extract_spotify_id("https://open.spotify.com/album/1DFixLWuPkv3KT3TnV35m3").unwrap();
        assert_eq!(content_type, "album");
        assert_eq!(id, "1DFixLWuPkv3KT3TnV35m3");
        
        let (content_type, id) = extract_spotify_id("https://open.spotify.com/playlist/37i9dQZF1DX4dyzvuaRJ0n").unwrap();
        assert_eq!(content_type, "playlist");
        assert_eq!(id, "37i9dQZF1DX4dyzvuaRJ0n");
        
        let (content_type, id) = extract_spotify_id("https://open.spotify.com/artist/4gzpq5DPGxSnKTe4SA8HAU").unwrap();
        assert_eq!(content_type, "artist");
        assert_eq!(id, "4gzpq5DPGxSnKTe4SA8HAU");
        
        // Test URI format
        let (content_type, id) = extract_spotify_id("spotify:track:4iV5W9uYEdYUVa79Axb7Rh").unwrap();
        assert_eq!(content_type, "track");
        assert_eq!(id, "4iV5W9uYEdYUVa79Axb7Rh");
        
        // Test invalid URLs
        assert!(extract_spotify_id("https://spotify.com/notavalidurl").is_err());
        assert!(extract_spotify_id("https://youtube.com/watch?v=1234").is_err());
    }
}