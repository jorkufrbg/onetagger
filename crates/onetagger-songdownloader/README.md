# OneTagger Song Downloader

A module for OneTagger that queries and downloads music from YouTube, Spotify, and SoundCloud channels, playlists, or individual tracks, automatically organizing them into folders.

## Features

- Extract songs from YouTube, Spotify, and SoundCloud sources
- Support for multiple URL formats:
  - YouTube: channels, videos, and playlists
  - Spotify: tracks, albums, playlists, and artists
  - SoundCloud: tracks and playlists
- Validates song names using proper formatting
- Downloads songs using yt-dlp or spotdl 
- Automatically organizes downloads into source-specific folders

## Supported URL Formats

### YouTube
- Channel URLs (`youtube.com/channel/[ID]`)
- Custom channel URLs (`youtube.com/c/[NAME]`)
- Handle URLs (`youtube.com/@[NAME]`)
- Playlist URLs (`youtube.com/playlist?list=[ID]`)
- Video URLs (`youtube.com/watch?v=[ID]`)

### Spotify
- Track URLs (`open.spotify.com/track/[ID]`)
- Album URLs (`open.spotify.com/album/[ID]`)
- Playlist URLs (`open.spotify.com/playlist/[ID]`)
- Artist URLs (`open.spotify.com/artist/[ID]`)
- Spotify URIs (`spotify:track:[ID]`, etc.)

### SoundCloud
- Track URLs (`soundcloud.com/[USER]/[TRACK]`)
- Playlist URLs (`soundcloud.com/[USER]/sets/[PLAYLIST]`)

## Usage

### Command Line Usage

```sh
# Query a URL and generate tracklist
onetagger query-url --url "https://www.youtube.com/@flavourtrip" --directory "C:\Users\Music\Flavour Trip"

# Download tracks from the generated tracklist
onetagger download-songs --json "C:\Users\Downloads\query-url\query-url.json" --directory "C:\Users\Music\Flavour Trip"
```

### Library Usage

```rust
use onetagger_songdownloader::{SongDownloader, get_url_info, download_songs_from_file};
use std::path::Path;

// Create a new downloader
let downloader = SongDownloader::new()
    .with_url("https://open.spotify.com/playlist/37i9dQZF1DX4dyzvuaRJ0n")
    .with_directory(Path::new("/path/to/download/folder"))
    .with_confidence(0.8)
    .with_output_format("json");

// Query the URL and generate output file
let output_file = downloader.query_url().unwrap();

// Download songs from the generated file
downloader.download_songs(&output_file).unwrap();
```

## Authentication

- **Spotify**: Uses existing authentication from the main OneTagger application. You must authenticate with Spotify in the main application before using the Spotify features of the song downloader.

## Technical Details

- YouTube scraping is done without using the YouTube API
- Spotify integration uses the official Spotify API via rspotify
- SoundCloud integration uses web scraping
- Downloads are handled by yt-dlp and spotdl

## Dependencies

- `yt-dlp` or `youtube-dl` for downloading songs
- `spotdl` as a fallback downloader