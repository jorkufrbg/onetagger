# Music Downloader

A script that queries and downloads music from YouTube / Spotify / Soundcloud channels or playlists, automatically organising them into folders.

## Features

- Extract songs from YouTube channels or playlists
- Supports multiple URL formats:
  - Channel URLs (`youtube.com/channel/[ID]`)
  - Custom channel URLs (`youtube.com/c/[NAME]`)
  - Handle URLs (`youtube.com/@[NAME]`)
  - Playlist URLs (`youtube.com/playlist?list=[ID]`)
- Validates song names using proper formatting
- Downloads songs using spotdl or yt-dlp (to be determined whats supported in Rust)
- Automatically organises downloads into channel/playlist-specific folders
- Optional ability to run Onetagger autotagger & audiofeatures for additional metadata after download completion.

## To build:

```sh
cd C:\Users\Matts-Pro\Documents\GitHub\onetagger
cargo build --release
```

### URL

- **Required**: Must be a YouTube channel, playlist, or video, or from Spotify or SoundCloud.
- **Validation**: URL must contain one of the following substrings: `youtube.com`, `youtu.be`, `spotify.com`, `soundcloud.com`.
- **Failure to provide**: Will throw an error and stop the script.

### Directory

- **Optional**: Directory where songs will be downloaded.
- **Validation**: If provided, it must exist. If it doesn't, print an error and stop the script.
- The directory is also used in `query-url` to check if a folder for a video already exists.

---

## To run a script for query-url

### To run a video:

```sh
.\target\release\onetagger.exe query-url --url "https://www.youtube.com/watch?v=2gq5Z83R6bQ" --directory "C:\Users\Matts-Pro\Music\Matts\Flavour Trip"
```

### To run a playlist:

```sh
.\target\release\onetagger.exe query-url --url "https://www.youtube.com/@flavourtrip/playlists"
```

### To run a channel:

```sh
.\target\release\onetagger.exe query-url --url "https://www.youtube.com/@flavourtrip"
```

---

### Channel Query Logic

- Detects a YouTube channel by presence of `@` in the URL.
- Example:

```sh
.\target\release\onetagger.exe query-url --url "https://www.youtube.com/@flavourtrip"
```

#### What happens for query-url

1. Converts input like `https://www.youtube.com/@flavourtrip` to `https://www.youtube.com/@flavourtrip/videos`.
2. Scrapes the channel for videos using a browser. Logs:  
   `Scraping Youtube channel Flavour Trip • 44 videos found.`
3. Loops through each video and logs:  
   `Scraping video 1 of 44`
4. For each video:
   - Visits the video page.
   - Extracts and logs the video title and description.
   - TODO Later: If no songs in description, than use SongRec (existing in onetagger repo) to extract songs from youtube video.
   - Example title:  
     `"Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set | Relaxing Sunset Dinner Playlist"`
   - Converts title into valid filename (removes invalid characters):  
     `Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set Relaxing Sunset Dinner Playlist`
   - Logs:  
     `Formatted video title: Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set Relaxing Sunset Dinner Playlist`
   - Checks if the formatted video title exists in the directory, and logs:  
     `Video title exists in directory: true`
     Else:
     `Video title exists in directory: false`
5. Take all songs from the directory and logs:
   - For each video: Use the songs to match songs in Youtube Music and add in metadata.
   - JSON output for all videos and songs. Have a true or false statement field for if user wants to download or not.

#### What happens for download-songs

DEPENDANCY on JSON output from query-url

```sh
.\target\release\onetagger.exe download-songs --json "C:\Users\Matts-Pro\Downloads\query-url\query-url.json" --directory "C:\Users\Matts-Pro\Music\Matts\Flavour Trip"
```

1. Takes the data and go through each video. If `Video title exists in directory: false` than create a folder with the formatted video title in the directory, followed by cd into the folder. If `Video title exists in directory: true` then skip creating folder and cd into the folder.
2. Filters out songs that have a downloaded flag. If `Downloaded: false` than remove song from the list.
3. Query each song in the list and check if it has a match in Youtube Music. If song match is greater than 0.5 match, than download each song using yt-dlp or spotdl (which should use only youtube music to download) and using yt-dlp or spotdl should already include the song's metadata. Don't use normal youtube, should only use youtube music in this step.
4. After video complete for downloaded songs, cd back to directory mentioned in variable. Move onto next video (Such as video 2), and repeat 1.
5. TODO Later: After completed all videos, and user mentioned optional variable run Onetagger autotagger & audiofeatures.

---

### Expected Regex for Tracklist Parsing

```regex
/^(?!Tracklist:)(?:(?:\d+\)\s*)?\d{1,2}:\d{2}(?::\d{2})?\s*)(._?)(?=\s_(?:\(|‪@|$))/gm
```

---

## Example Video Description Parsing

Video source: [YouTube Link](https://www.youtube.com/watch?v=hLEmDqmcCzM)

#### Scraped data (partial example):

```txt
Video 1: Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set Relaxing Sunset Dinner Playlist - Premiered on 19 Mar 2025 - 627,585 views - https://www.youtube.com/watch?v=hLEmDqmcCzM
25 tracks found

1. 00:00 Milan93 - Just To Relax (Cabriolet)‪@theBasementDiscos‬
2. 03:09 Baka G - Delta Leonids ‪@ht_rec‬
3. 06:57 ColorJaxx - When You Find ‪@Flipsight‬
4. 10:44 Stogov & Gilista - Sunset Mood ‪@lisztomaniarec‬
...
```

- Wait for `#description-inner` element in HTML.
- Extract with JavaScript:

  ```js
  document.querySelector("#description-inner");
  ```

Step 1: It will take the variable url and look for the channel, and edit the url. For example, if input is https://www.youtube.com/@flavourtrip it changes it it https://www.youtube.com/@flavourtrip/videos
Step 2: It will take the edited url and go to a browser and scrape the videos from the channel. Print a log saying, "Scraping Youtube channel Flavour Trip • 44 videos found."
Step 3: Go through each videos from that channel, and print a log saying, "Scraping video 1 of 44"
Step 4: For each video, it will take the video url (example https://www.youtube.com/watch?v=hLEmDqmcCzM) and go to a browser and scrape the video title (example "Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set | Relaxing Sunset Dinner Playlist") and video description. Example of the video title and description is below.

Print a log saying, "Scraping video 1 of

get all videos, for example for above example its currently 44 videos and provide a log like
and download all the songs in that playlist.
Regex /^(?!Tracklist:)(?:(?:\d+\)\s*)?\d{1,2}:\d{2}(?::\d{2})?\s*)(._?)(?=\s_(?:\(|‪@|$))/gm

Example of a youtube video description we are expecting from https://www.youtube.com/watch?v=hLEmDqmcCzM. We would wait till we have "#description-inner" in the HTML. Then we use the JS Path: document.querySelector("#description-inner")

[
627,585 views Premiered on 20 Mar 2025 1 product
Tracklist down below👇🏼
Flavour Trip Hoodies available on https://flavourtrip.com/collections/all

Meet us in a private video call, get access to behind the scenes, recipes, polls and more on / flavourtrip

In this set we're playing relaxing deep house with a view over the old town of Tarragona. Winter is slowly fading away, the birds start to sing and spring is close. Let the smooth house beats heat us up and let's watch the sun turn golden. Put on your dancing shoes, grab your favourite snack and join the trip!
We're happy that you exist, the world needs you!

Tracklist:

1. 00:00 Milan93 - Just To Relax (Cabriolet) @theBasementDiscos‬
2. 03:09 Baka G - Delta Leonids ‪@ht_rec‬
3. 06:57 ColorJaxx - When You Find ‪@Flipsight‬
4. 10:44 Stogov & Gilista - Sunset Mood ‪@lisztomaniarec‬
5. 15:05 Sebb Junior - A Piece Of Me ‪@suburbanmusictv‬
6. 20:20 Le Hutin, Lay - I Hear ‘Em Voices ‪@BasicsRecording‬
7. 24:47 Scott Diaz -In These Stars ‪@suburbanmusictv‬
8. 29:57 Mindeliq - Stray Cats (55 Music)
9. 32:36 Dynamique - Last Sunset (Cabriolet)‪@theBasementDiscos‬
10. 35:05 Igor Gonya & Bauhouse - No Smoke ‪@lisztomaniarec‬
11. 39:25 Saison - What Are We Gonna Do ‪@nofussrecords6609‬
12. 44:08 Brous One & Felipe Gordon - Right Away (55 Music)
13. 48:55 øverfeel - I will be okay (55 Music)
14. 53:41 Paolo Barbato, Lee Wilson - Coffee Date (Sift)‪@nofussrecords6609‬
15. 59:02 Brooklyn Baby - For Your Eyes Only ‪@hustlertrax6841‬
16. 01:02:50 Green Sequence - Easy Love ‪@lisztomaniarec‬
17. 01:05:56 Homero Espinosa - Carry On ‪@MoultonMusic‬
18. 01:11:27 T.Markakis - Let’s Get It On ‪@moissmusic9275‬
19. 01:15:47 Cpen, JT Donaldson - Just Can’t ‪@nofussrecords6609‬
20. 01:21:15 weS!, Wei F. - Gone wit the sWIND (Closer To Truth)
21. 01:26:50 Nicola Nisi - Jazz Plastik (Plastik People)
22. 01:31:07 Dylan Dylan - Do You Need Me? ‪@PontNeufRecords‬
23. 01:36:57 Ridney - Outta My Mind ‪@PaharasMusica‬
24. 01:41:00 Noise Ark - 90’s Rave ‪@ht_rec‬
25. 01:45:19 Melon Bomb - Just Peachy ‪@theBasementDiscos‬

flavourful timestamps:
02:25 Cheers!
11:06 cutting Manchego cheese
29:59 Amii feeling the vibes
40:40 Can you see the bear?
48:16 jazzy tour
59:31 birds are enjoying the sunset too
01:07:50 wow
01:09:45 Tarragona slowly calming down
01:14:12 the back bender
1:32:18 sing along even if you don't know the lyrics
01:38:56 Love is the answer!
01:45:59 above the city lights

Follow Flavour Trip:
Instagram: / flavourtripmusic  
Spotify Playlists: https://open.spotify.com/user/315jg75...
Soundcloud: / flavourtrip  
Patreon: / flavourtrip
]

Expected Extract and format video title.
Convert video title into what is allowed for a Windows or MacOS file name. Remove all characters that are not allowed for a Windows or MacOS. Example "Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set | Relaxing Sunset Dinner Playlist" gets converted into "Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set Relaxing Sunset Dinner Playlist".
Logs than state "Formatted video title: Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set Relaxing Sunset Dinner Playlist".

Expected Extract and format video description.
For example, using the above video description, expected output is:
[
Video 1: Soft Rooftop House Music Mix - Chillout Afterwork Lounge Set Relaxing Sunset Dinner Playlist - Premiered on 19 Mar 2025 - 627,585 views - https://www.youtube.com/watch?v=hLEmDqmcCzM
25 tracks found

1. Milan93 - Just To Relax
2. Baka G - Delta Leonids
3. ColorJaxx - When You Find
4. Stogov & Gilista - Sunset Mood
5. Sebb Junior - A Piece Of Me
6. Le Hutin, Lay - I Hear ‘Em Voices
7. Scott Diaz -In These Stars
8. Mindeliq - Stray Cats
9. Dynamique - Last Sunset
10. Igor Gonya & Bauhouse - No Smoke
11. Saison - What Are We Gonna Do
12. Brous One & Felipe Gordon - Right Away
13. øverfeel - I will be okay
14. Paolo Barbato, Lee Wilson - Coffee Date
15. Brooklyn Baby - For Your Eyes Only
16. Green Sequence - Easy Love
17. Homero Espinosa - Carry On
18. T.Markakis - Let’s Get It On
19. Cpen, JT Donaldson - Just Can’t
20. weS!, Wei F. - Gone wit the sWIND
21. Nicola Nisi - Jazz Plastik
22. Dylan Dylan - Do You Need Me?
23. Ridney - Outta My Mind
24. Noise Ark - 90’s Rave
25. Melon Bomb - Just Peachy
    ]

Rules:

1. No YouTube API usage - uses web scraping instead
2. No browser extensions or ChromeDriver - uses reqwest and scraper libraries
3. No test files or static output - all data is dynamically processed
