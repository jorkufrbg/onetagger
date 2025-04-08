# Song Downloader Documentation

## Building the Project

```bash
cd C:\Users\Matts-Pro\Documents\GitHub\onetagger
cargo build --release
```

## Parameter Requirements

### URL Parameter

- **Required**: Yes
- **Valid Sources**:
  - YouTube (videos, channels, playlists)
  - Spotify
  - SoundCloud
- **Validation Rules**:
  - Must contain one of: `youtube.com`, `1001tracklists.com`, `youtu.be`, `spotify.com`, `soundcloud.com`
  - Script will terminate if validation fails

### Directory Parameter

- **Optional**: Yes
- **Validation**:
  - Must exist if provided
  - Script will terminate if directory doesn't exist
- **Usage**:
  - Used for downloading songs
  - Used in `query-url` to check for existing video folders

## Usage Examples

### Download Single Video

```bash
.\target\release\onetagger.exe query-url --url "https://www.youtube.com/watch?v=2gq5Z83R6bQ" --directory "C:\Users\Matts-Pro\Music\Matts\Flavour Trip"
```

### Process Playlist

```bash
.\target\release\onetagger.exe query-url --url "https://www.youtube.com/@flavourtrip/playlists"
```

### Process Channel

```bash
.\target\release\onetagger.exe query-url --url "https://www.youtube.com/@flavourtrip"
```

## Channel Processing Logic

1. **URL Processing**

   - Detects channel by `@` in URL
   - Transforms URL (e.g., `@flavourtrip` → `@flavourtrip/videos`)

2. **Scraping Process**

   - Scrapes channel content using browser
   - Logs: `Scraping Youtube channel Flavour Trip • 44 videos found`
   - Progress updates: `Scraping video 1 out of 44`

3. **Per Video Processing**
   - Visits each video page
   - Extracts title and description
   - Sanitizes filename (removes invalid characters)
   - Example transformation:
     - Original: `"Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set | Relaxing Sunset Dinner Playlist"`
     - Sanitized: `"Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set Relaxing Sunset Dinner Playlist"`

## Technical Details

### Tracklist Parsing Regex

```regex
/^(?!Tracklist:)(?:(?:\d+\)\s*)?\d{1,2}:\d{2}(?::\d{2})?\s*)(._?)(?=\s_(?:\(|‪@|$))/gm
```

### HTML Extraction

- Waits for `#description-inner` element
- JavaScript selector:

```javascript
document.querySelector("#description-inner");
```

## Example Output Format

```plaintext
Video 1: Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set Relaxing Sunset Dinner Playlist
Premiered on 19 Mar 2025 - 627,585 views
URL: https://www.youtube.com/watch?v=hLEmDqmcCzM
25 tracks found

1. 00:00 Milan93 - Just To Relax
2. 03:09 Baka G - Delta Leonids
3. 06:57 ColorJaxx - When You Find
[...]
25. Melon Bomb - Just Peachy
```

## Important Rules

1. **No YouTube API**

   - Must use browser-based scraping
   - Direct API usage not allowed

2. **No Browser Extensions/ChromeDriver**
   - Must be user-friendly
   - Must be scalable for public use
   - No external browser automation tools allowed
