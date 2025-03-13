use std::collections::HashMap;
use anyhow::Error;
use axum::extract::ws::{WebSocket, Message};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use onetagger_renamer::ac::Autocomplete;
use onetagger_renamer::docs::FullDocs;
use onetagger_renamer::{Renamer, TemplateParser, RenamerConfig};
use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use dunce::canonicalize;
use onetagger_tag::{TagChanges, TagSeparators, Tag, Field};
use onetagger_tagger::{TaggerConfig, AudioFileInfo, TrackMatch};
use onetagger_autotag::{Tagger, AudioFileInfoImpl, TaggerConfigExt, AUTOTAGGER_PLATFORMS};
use onetagger_autotag::audiofeatures::{AudioFeaturesConfig, AudioFeatures};
use onetagger_platforms::spotify::Spotify;
use onetagger_player::{AudioSources, AudioPlayer};
use onetagger_shared::{Settings, COMMIT};
use onetagger_playlist::{UIPlaylist, PLAYLIST_EXTENSIONS, get_files_from_playlist_file};
use std::process::Command;
use std::thread;
use crossbeam_channel::unbounded;
use std::env;
use std::fs;

use crate::StartContext;
use crate::quicktag::{QuickTag, QuickTagFile, QuickTagData};
use crate::tageditor::TagEditor;
use crate::browser::{FileBrowser, FolderBrowser};


#[derive(Debug, Clone, Serialize, Deserialize)]
struct FoundSong {
    title: String,
    artist: String,
    video_url: String,
    timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
enum Action {
    Init,
    Exit,
    SaveSettings { settings: Value },
    LoadSettings,
    DefaultCustomPlatformSettings,
    Browse { path: Option<String>, context: Option<String> },
    Browser { url: String },
    OpenSettingsFolder,
    OpenFolder { path: PathBuf },
    OpenFile { path: PathBuf },
    DeleteFiles { paths: Vec<String> },
    GetLog,
    GeneratePlaylist { paths: Vec<String> },

    LoadPlatforms,
    StartTagging { config: TaggerConfigs, playlist: Option<UIPlaylist> },
    StopTagging,
    ConfigCallback { config: Value, platform: String, id: String },
    RepoManifest,
    #[serde(rename_all = "camelCase")]
    InstallPlatform { id: String, version: String, is_native: bool },

    Waveform { path: PathBuf },
    PlayerLoad { path: PathBuf },
    PlayerPlay, 
    PlayerPause,
    PlayerSeek { pos: u64 },
    PlayerVolume { volume: f32 },
    PlayerStop,

    QuickTagLoad { path: Option<String>, playlist: Option<UIPlaylist>, recursive: Option<bool>, separators: TagSeparators, limit: Option<bool> },
    QuickTagSave { changes: TagChanges },
    QuickTagFolder { path: Option<String>, subdir: Option<String> },

    #[serde(rename_all = "camelCase")]
    SpotifyAuthorize { client_id: String, client_secret: String },
    SpotifyAuthorized,

    TagEditorFolder { path: Option<String>, subdir: Option<String>, recursive: Option<bool>  },
    TagEditorLoad { path: PathBuf },
    TagEditorSave { changes: TagChanges },

    RenamerSyntaxHighlight { template: String },
    RenamerAutocomplete { template: String },
    RenamerPreview { config: RenamerConfig },
    RenamerStart { config: RenamerConfig },

    FolderBrowser { path: PathBuf, child: String, base: bool },

    ManualTag { config: TaggerConfig, path: PathBuf },
    ManualTagApply { matches: Vec<TrackMatch>, path: PathBuf, config: TaggerConfig },
    
    #[serde(rename_all = "camelCase")]
    AnalyzeSongs { url: String, confidence: f32 },
    
    #[serde(rename_all = "camelCase")]
    DownloadSongs { 
        url: String, 
        output_path: String, 
        confidence: f32, 
        enable_auto_tag: bool, 
        auto_tag_config: Option<String>,
        enable_audio_features: bool,
        songs: Vec<FoundSong>
    },
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
enum TaggerConfigs {
    AutoTagger(TaggerConfig), 
    AudioFeatures(AudioFeaturesConfig)
}

impl TaggerConfigs {
    // Print to log for later easier debug
    pub fn debug_print(&self) {
        match self {
            TaggerConfigs::AutoTagger(c) => {
                let mut c = c.clone();
                // don't leak secrets
                c.custom = HashMap::new().into();
                c.spotify = None;
                info!("AutoTagger config: {:?}", c);
            },
            TaggerConfigs::AudioFeatures(c) => {
                info!("AudioFeatures Config: {:?}", c);
            }
        }
    }
}

// Shared variables in socket
struct SocketContext {
    player: AudioPlayer,
    spotify: Option<Spotify>,
    start_context: StartContext
} 

impl SocketContext {
    pub fn new(start_context: StartContext) -> SocketContext {
        SocketContext {
            player: AudioPlayer::new(),
            spotify: None,
            start_context
        }
    }
}


/// Reply to init call
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitData {
    action: &'static str,
    version: &'static str,
    os: &'static str,
    arch: &'static str,
    custom_platform_compat: i32,
    start_context: StartContext,
    renamer_docs: FullDocs,
    commit: &'static str,
    work_dir: PathBuf,
    data_dir: PathBuf
}

impl InitData {
    /// Create new default instance
    pub fn new(start_context: StartContext) -> InitData {
        InitData {
            action: "init",
            version: onetagger_shared::VERSION,
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            custom_platform_compat: onetagger_tagger::custom::CUSTOM_PLATFORM_COMPATIBILITY,
            start_context,
            renamer_docs: FullDocs::get().html(),
            commit: COMMIT,
            work_dir: std::env::current_dir().unwrap_or_default(),
            data_dir: Settings::get_folder().unwrap_or_default(),
        }
    }
}

pub(crate) async fn handle_ws_connection(mut websocket: WebSocket, context: StartContext) -> Result<(), Error> {
    let mut context = SocketContext::new(context);
    
    while let Some(message) = websocket.recv().await {
        match message {
            Ok(msg) => {
                match msg.to_text() {
                    Ok(text) => {
                        // Handle the WS message
                        match handle_message(text, &mut websocket, &mut context).await {
                            Ok(_) => {},
                            Err(err) => {
                                // Send error to UI
                                error!("Websocket: {:?}, Data: {}", err, text);
                                send_socket(&mut websocket, json!({
                                    "action": "error",
                                    "message": &format!("{}", err)
                                })).await.ok();
                            }
                        }
                    },
                    Err(e) => warn!("WebSocket Message is not text: {e}"),
                }
            }

            Err(e) => {
                warn!("WebSocket error: {e}");
            }
        }
    
    }

    Ok(())
}

/// Serialize and send to socket with warning intercept
async fn send_socket<D: Serialize>(ws: &mut WebSocket, json: D) -> Result<(), Error> {
    match send_socket_inner(ws, json).await {
        Ok(_) => Ok(()),
        Err(e) => {
            warn!("Failed sending to socket: {e}");
            Err(e)
        },
    }
}

/// Serialize and send to socket
async fn send_socket_inner<D: Serialize>(ws: &mut WebSocket, json: D) -> Result<(), Error> {
    ws.send(Message::from(serde_json::to_string(&json)?)).await?;
    Ok(())
}

/// Analyze a YouTube URL to find songs
fn analyze_songs(url: &str, confidence: f32) -> Result<Vec<FoundSong>, Error> {
    // Get the path to the Python script
    let script_path = env::current_dir()?
        .join("YoutubeToSpotify")
        .join("analyzer.py");
    
    // Check if the script exists
    if !script_path.exists() {
        return Err(anyhow::anyhow!("Song analyzer script not found at {:?}", script_path));
    }
    
    // Prepare the command
    let output = Command::new("python")
        .arg(&script_path)
        .arg("--url").arg(url)
        .arg("--confidence").arg(confidence.to_string())
        .arg("--mode").arg("analyze")
        .output()?;
    
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to analyze songs: {}", error_message));
    }
    
    // Parse the output JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let songs: Vec<FoundSong> = serde_json::from_str(&stdout)?;
    
    Ok(songs)
}

/// Download songs from a YouTube URL
fn download_songs(
    url: &str,
    output_path: &str,
    confidence: f32,
    enable_auto_tag: bool,
    auto_tag_config: Option<String>,
    enable_audio_features: bool,
    songs: &[FoundSong]
) -> Result<(), Error> {
    // Create the output directory if it doesn't exist
    fs::create_dir_all(output_path)?;
    
    // Get the path to the Python script
    let script_path = env::current_dir()?
        .join("YoutubeToSpotify")
        .join("downloader.py");
    
    // Check if the script exists
    if !script_path.exists() {
        return Err(anyhow::anyhow!("Song downloader script not found at {:?}", script_path));
    }
    
    // Create a temporary JSON file with song information
    let songs_json = serde_json::to_string(songs)?;
    let temp_songs_file = env::temp_dir().join("onetagger_songs.json");
    fs::write(&temp_songs_file, &songs_json)?;
    
    // Build the command
    let mut cmd = Command::new("python");
    cmd.arg(&script_path)
        .arg("--url").arg(url)
        .arg("--output").arg(output_path)
        .arg("--confidence").arg(confidence.to_string())
        .arg("--songs-file").arg(&temp_songs_file);
    
    // Add optional flags
    if enable_auto_tag {
        cmd.arg("--enable-auto-tag");
        if let Some(config) = &auto_tag_config {
            let config_path = env::temp_dir().join("onetagger_autotagconfig.json");
            fs::write(&config_path, config)?;
            cmd.arg("--auto-tag-config").arg(&config_path);
        }
    }
    
    if enable_audio_features {
        cmd.arg("--enable-audio-features");
    }
    
    // Run the command
    let output = cmd.output()?;
    
    // Clean up temporary files
    if temp_songs_file.exists() {
        fs::remove_file(temp_songs_file).ok();
    }
    
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to download songs: {}", error_message));
    }
    
    Ok(())
}

async fn handle_message(text: &str, websocket: &mut WebSocket, context: &mut SocketContext) -> Result<(), Error> {
    // Parse JSON
    let action: Action = serde_json::from_str(text)?;
    match action {
        // Get initial info
        Action::Init => {
            send_socket(websocket, InitData::new(context.start_context.clone())).await.ok();
        },
        Action::Exit => std::process::exit(0),
        Action::SaveSettings { settings } => Settings::from_ui(&settings).save()?,
        Action::LoadSettings => match Settings::load() {
            Ok(settings) => {
                send_socket(websocket, json!({
                    "action": "loadSettings",
                    "settings": settings.ui
                })).await.ok();
            }
            // Ignore settings if they don't exist (might be initial load)
            Err(e) => error!("Failed loading settings, using defaults. {}", e)
        },
        // Get the default custom platform options
        Action::DefaultCustomPlatformSettings => {
            send_socket(websocket, json!({
                "action": "defaultCustomPlatformSettings",
                "custom": TaggerConfig::custom_default().custom
            })).await.ok();
        }
        // Browse for folder
        Action::Browse { path, context } => {
            let mut initial = path.unwrap_or(".".to_string());
            if initial.is_empty() || !Path::new(&initial).exists() {
                initial = ".".to_string()
            }
            if let Some(path) = tinyfiledialogs::select_folder_dialog("Select path", &initial) {
                send_socket(websocket, json!({
                    "action": "browse",
                    "path": path,
                    "context": context
                })).await.ok();
            }
        },
        // Get 1t Log
        Action::GetLog => {
            log::logger().flush();
            let log = std::fs::read_to_string(&Settings::get_folder()?.join("onetagger.log"))?;
            send_socket(websocket, json!({
                "action": "log",
                "log": log
            })).await.ok();
        },
        // Open URL in external browser
        Action::Browser { url } => { webbrowser::open(&url)?; },
        Action::OpenSettingsFolder => opener::open(Settings::get_folder()?.to_str().unwrap())?,
        Action::OpenFolder { path } => { opener::open(&path).ok(); },
        Action::OpenFile { path } => { opener::open(&path).ok(); },
        Action::DeleteFiles { paths } => { trash::delete_all(&paths)?; },

        Action::GeneratePlaylist { paths } => {
            let playlist = onetagger_playlist::create_m3u_playlist(&paths.into_iter().map(|i| i.into()).collect::<Vec<_>>());
            if let Some(path) = tinyfiledialogs::save_file_dialog_with_filter(
                "Save playlist", 
                &std::env::current_dir()?.to_string_lossy().to_string(), 
                &["m3u", "m3u8"], 
                "Save playlist"
            ) {
                std::fs::write(&path, playlist)?;
                send_socket(websocket, json!({
                    "action": "notify",
                    "message": format!("Playlist saved to: {path}")
                })).await.ok();
            }
        }

        Action::LoadPlatforms => {
            let platforms = tokio::task::spawn_blocking(|| {
                let mut platforms = AUTOTAGGER_PLATFORMS.lock().unwrap();
                platforms.load_all();
                platforms.platforms.iter().map(|p| p.info.clone()).collect::<Vec<_>>()
            }).await?;
            send_socket(websocket, json!({
                "action": "loadPlatforms",
                "platforms": platforms
            })).await.ok();
        },
        Action::ConfigCallback { config, platform, id } => {
            let platform_clone = platform.clone();
            let response = tokio::task::spawn_blocking(move || {
                if let Some(p) = AUTOTAGGER_PLATFORMS.lock().unwrap().get_builder(&platform) {
                    Some(p.config_callback(&id, config))
                } else {
                    None
                }
            }).await?;
            if let Some(r) = response {
                send_socket(websocket, json!({
                    "action": "configCallback",
                    "platform": platform_clone,
                    "response": r
                })).await.ok();
            }
        }
        Action::StartTagging { config, playlist } => {
            config.debug_print();

            // Load playlist
            let mut files = if let Some(playlist) = playlist {
                playlist.get_files()?
            } else { vec![] };
            let mut file_count = files.len();
            let mut folder_path = None;
            let tagger_finished = Arc::new(Mutex::new(None));
            // Load taggers
            let (tagger_type, rx) = match config {
                TaggerConfigs::AutoTagger(c) => {
                    // Load file list
                    if files.is_empty() {
                        let path = c.path.as_ref().map(|p| p.to_owned()).unwrap_or_default();
                        files = AudioFileInfo::get_file_list(&path, c.include_subfolders);
                        file_count = files.len();
                        folder_path = Some(path);
                    }
                    let rx = Tagger::tag_files(&c, files, tagger_finished.clone());
                    ("autoTagger", rx)
                },
                TaggerConfigs::AudioFeatures(c) => {
                    if files.is_empty() {
                        let path = c.path.as_ref().map(|i| i.to_owned()).unwrap_or_default().to_owned();
                        files = AudioFileInfo::get_file_list(&path, c.include_subfolders);
                        folder_path = Some(path);
                        file_count = files.len();
                    }
                    // Authorize spotify
                    let spotify = context.spotify.as_ref().ok_or(anyhow!("Spotify unauthorized!"))?.to_owned().to_owned();
                    let rx = AudioFeatures::start_tagging(c.clone(), spotify, files);
                    ("audioFeatures", rx)
                },
            };

            // Start
            let start = timestamp!();
            send_socket(websocket, json!({
                "action": "startTagging",
                "files": file_count,
                "type": tagger_type
            })).await.ok();
            // Tagging
            for status in rx {
                send_socket(websocket, json!({
                    "action": "taggingProgress",
                    "status": status
                })).await.ok();
            }
            info!("Tagging finished, took: {} seconds.", (timestamp!() - start) / 1000);
            // Done
            send_socket(websocket, json!({
                "action": "taggingDone",
                "path": folder_path,
                "data": *tagger_finished.lock().unwrap()
            })).await.ok();
        },
        Action::StopTagging => {
            onetagger_autotag::STOP_TAGGING.store(true, Ordering::SeqCst);
        },
        Action::Waveform { path } => {
            let source = AudioSources::from_path(&path)?;
            let (waveform_rx, cancel_tx) = source.generate_waveform(180)?;
            // Streamed
            for wave in waveform_rx {
                send_socket(websocket, json!({
                    "action": "waveformWave",
                    "wave": wave
                })).await.ok();
                // Check reply
                if websocket.recv().await.is_none() {
                    cancel_tx.send(true).ok();
                }
            }
            // Done
            send_socket(websocket, json!({
                "action": "waveformDone",
            })).await.ok();
        },
        // Load player file
        Action::PlayerLoad { path } => {
            let source = AudioSources::from_path(&path)?;
            // Meta
            let tag = Tag::load_file(&path, false)?;
            let title = tag.tag().get_field(Field::Title).map(|i| i.first().map(String::from)).flatten();
            let artists = tag.tag().get_field(Field::Artist).unwrap_or(vec![]);
            // Send to UI
            send_socket(websocket, json!({
                "action": "playerLoad",
                "title": title,
                "artists": artists,
                "duration": source.duration() as u64
            })).await.ok();
            // Load
            context.player.load_file(source);
        },
        //  Controls
        Action::PlayerPlay => context.player.play(),
        Action::PlayerPause => context.player.pause(),
        Action::PlayerSeek { pos } => {
            send_socket(websocket, json!({
                "action": "playerSync",
                "playing": context.player.seek(pos)
            })).await.ok();
        },
        Action::PlayerVolume { volume } => context.player.volume(volume),
        Action::PlayerStop => context.player.stop(),
        // Load quicktag files or playlist
        Action::QuickTagLoad { path, playlist, recursive, separators, limit } => {
            let mut data = QuickTagData::default();
            // Playlist
            if let Some(playlist) = playlist {
                data = QuickTag::load_files_playlist(&playlist, &separators)?;
            }
            // Path
            if let Some(path) = path {
                if PLAYLIST_EXTENSIONS.iter().any(|e| path.to_lowercase().ends_with(e)) {
                    data = QuickTag::load_files(get_files_from_playlist_file(&path)?, &separators)?;
                } else {
                    data = QuickTag::load_files_path(
                        &path, 
                        recursive.unwrap_or(false), 
                        &separators, 
                        0, 
                        limit.map(|l| l.then_some(500)).flatten().unwrap_or(usize::MAX)
                    )?;
                }
            }
            send_socket(websocket, json!({
                "action": "quickTagLoad",
                "data": data
            })).await.ok();
        },
        // Save quicktag changes
        Action::QuickTagSave { changes } => {
            let tag = changes.commit()?;
            send_socket(websocket, json!({
                "action": "quickTagSaved",
                "path": &changes.path,
                "file": QuickTagFile::from_tag(&changes.path, &tag)?
            })).await.ok();
        },
        // List dir
        Action::QuickTagFolder { path, subdir } => {
            let (new_path, files) = FileBrowser::list_dir_or_default(path.clone().map(|p| PathBuf::from(p)), subdir, true, false, false)?;
            send_socket(websocket, json!({
                "action": "quickTagFolder",
                "files": files,
                "path": new_path,
            })).await.ok();
        }
        Action::SpotifyAuthorize { client_id, client_secret } => {
            // Authorize cached
            if let Some(spotify) = Spotify::try_cached_token(&client_id, &client_secret) {
                context.spotify = Some(spotify);
            // Authorize new
            } else {
                let (auth_url, client) = Spotify::generate_auth_url(&client_id, &client_secret)?;
                webbrowser::open(&auth_url)?;
                let spotify = tokio::task::spawn_blocking(move || {
                    Spotify::auth_server(client)
                }).await??;
                context.spotify = Some(spotify);
            }
            send_socket(websocket, json!({
                "action": "spotifyAuthorized",
                "value": true
            })).await.ok();
            debug!("Spotify Authorized!");
        },
        // Check if authorized
        Action::SpotifyAuthorized => {
            send_socket(websocket, json!({
                "action": "spotifyAuthorized",
                "value": context.spotify.is_some()
            })).await.ok();
        },
        Action::TagEditorFolder { path, subdir, recursive } => {
            let recursive = recursive.unwrap_or(false);
            let (new_path, files) = FileBrowser::list_dir_or_default(path.clone().map(|p| PathBuf::from(p)), subdir, true, true, recursive)?;
            send_socket(websocket, json!({
                "action": "tagEditorFolder",
                "files": files,
                "path": new_path,
                // Stateless
                "recursive": recursive
            })).await.ok();
        },
        // Load tags of file
        Action::TagEditorLoad { path } => {
            let data = TagEditor::load_file(&path)?;
            send_socket(websocket, json!({
                "action": "tagEditorLoad",
                "data": data
            })).await.ok();
        },
        // Save changes
        Action::TagEditorSave { changes } => {
            let _tag = changes.commit()?;
            send_socket(websocket, json!({
                "action": "tagEditorSave"
            })).await.ok();
        },
        // Syntax highlight for renamer
        Action::RenamerSyntaxHighlight { template } => {
            let renamer = Renamer::new(TemplateParser::parse(&template));
            let html = renamer.generate_html(&template);
            send_socket(websocket, json!({
                "action": "renamerSyntaxHighlight",
                "html": html
            })).await.ok();
        },
        // Autocomplete data
        Action::RenamerAutocomplete { template } => {
            let ac = Autocomplete::parse(&template);
            let suggestions = ac.suggest_html();
            send_socket(websocket, json!({
                "action": "renamerAutocomplete",
                "suggestions": suggestions,
                "offset": ac.suggestion_offset()
            })).await.ok();
        },
        // Generate new names but don't rename
        Action::RenamerPreview { config } => {
            let mut renamer = Renamer::new(TemplateParser::parse(&config.template));
            let files = AudioFileInfo::load_files_iter(&config.path, config.subfolders, None, None);
            let files = renamer.generate(files.take(3), &config).unwrap_or(vec![]);
            send_socket(websocket, json!({
                "action": "renamerPreview",
                "files": files,
            })).await.ok();
        },
        // Start renamer
        Action::RenamerStart { config } => {
            let mut renamer = Renamer::new(TemplateParser::parse(&config.template));
            let files = AudioFileInfo::load_files_iter(&config.path, config.subfolders, None, None);
            let files = renamer.generate(files, &config)?;
            renamer.rename(&files, &config)?;
            send_socket(websocket, json!({
                "action": "renamerDone",
            })).await.ok();
        },
        // File browser list dir
        Action::FolderBrowser { path, child , base } => {
            // Windows root dir override
            let path = if cfg!(windows) && path.to_string_lossy() == "/" {
                if child.is_empty() {
                    PathBuf::from("/".to_string())
                } else {
                    PathBuf::from(format!("{}\\", child))
                }
            } else {
                canonicalize(PathBuf::from(path).join(child))?
            };

            let e = match base {
                true => FolderBrowser::generate_base(&path)?,
                false => FolderBrowser::list_dir(&path)?
            };

            send_socket(websocket, json!({
                "action": "folderBrowser",
                "entry": e,
                "base": base,
                "path": path
            })).await.ok();
        },

        // Manually tag a file
        Action::ManualTag { config, path } => {
            // Log config
            info!("Manual tag starting for path: {path:?}");
            TaggerConfigs::AutoTagger(config.clone()).debug_print();

            let rx = tokio::task::spawn_blocking(move || {
                onetagger_autotag::manual_tagger(path, &config)
            }).await.unwrap()?;

            for (platform, r) in rx {
                match r {
                    Ok(matches) => {
                        send_socket(websocket, json!({
                            "action": "manualTag",
                            "platform": platform,
                            "status": "ok",
                            "matches": matches
                        })).await.ok();
                    },
                    Err(e) => {
                        send_socket(websocket, json!({
                            "action": "manualTag",
                            "platform": platform,
                            "status": "error",
                            "error": e.to_string()
                        })).await.ok();
                    },
                }
            }

            // On done
            send_socket(websocket, json!({
                "action": "manualTagDone"
            })).await.ok();
        },
        // Apply the tags from manual tagger
        Action::ManualTagApply { matches, path, config } => {
            match onetagger_autotag::manual_tagger_apply(matches, path, &config) {
                Ok(_) => {
                    send_socket(websocket, json!({
                        "action": "manualTagApplied",
                        "status": "ok"
                    })).await.ok();
                },
                Err(e) => {
                    error!("Failed applying manual tag: {e}");
                    send_socket(websocket, json!({
                        "action": "manualTagApplied",
                        "status": "error",
                        "error": e.to_string()
                    })).await.ok();
                },
            }
        },

        Action::AnalyzeSongs { url, confidence } => {
            info!("Analyzing YouTube URL: {}", url);
            
            // Process in a background thread
            let (tx, rx) = unbounded();
            
            // Clone values for thread
            let url_clone = url.clone();
            
            thread::spawn(move || {
                match analyze_songs(&url_clone, confidence) {
                    Ok(songs) => tx.send(Ok(songs)).ok(),
                    Err(e) => tx.send(Err(e)).ok(),
                }
            });
            
            // Wait for result from background thread
            if let Ok(result) = rx.recv() {
                match result {
                    Ok(songs) => {
                        send_socket(websocket, json!({
                            "action": "analyzeSongs",
                            "songs": songs
                        })).await.ok();
                    },
                    Err(e) => {
                        error!("Failed analyzing songs: {}", e);
                        send_socket(websocket, json!({
                            "action": "analyzeSongs",
                            "error": e.to_string()
                        })).await.ok();
                    }
                }
            }
        },
        
        Action::DownloadSongs { url, output_path, confidence, enable_auto_tag, auto_tag_config, enable_audio_features, songs } => {
            info!("Starting song download process from URL: {}", url);
            
            // Start a background thread to handle the download
            let (tx, rx) = unbounded();
            
            // Clone values for thread
            let url_clone = url.clone();
            let output_path_clone = output_path.clone();
            let songs_clone = songs.clone();
            let auto_tag_config_clone = auto_tag_config.clone();
            
            thread::spawn(move || {
                let result = download_songs(
                    &url_clone, 
                    &output_path_clone, 
                    confidence, 
                    enable_auto_tag, 
                    auto_tag_config_clone,
                    enable_audio_features, 
                    &songs_clone
                );
                tx.send(result).ok();
            });
            
            // Wait for result
            if let Ok(result) = rx.recv() {
                match result {
                    Ok(_) => {
                        send_socket(websocket, json!({
                            "action": "downloadSongs",
                            "success": true
                        })).await.ok();
                    },
                    Err(e) => {
                        error!("Failed downloading songs: {}", e);
                        send_socket(websocket, json!({
                            "action": "downloadSongs",
                            "success": false,
                            "error": e.to_string()
                        })).await.ok();
                    }
                }
            }
        },

        Action::RepoManifest => {
            send_socket(websocket, json!({
                "action": "repoManifest",
                "manifest": onetagger_autotag::repo::fetch_manifest_async().await?
            })).await.ok();
        },
        Action::InstallPlatform { id, version, is_native } => {
            match onetagger_autotag::repo::install_platform(&id, &version, is_native) {
                Ok(_) => send_socket(websocket, json!({
                    "action": "installPlatform",
                    "status": "ok"
                })).await.ok(),
                Err(e) => {
                    error!("Failed installing platform {id}@{version}: {e}");
                    send_socket(websocket, json!({
                        "action": "installPlatform",
                        "status": "error",
                        "error": e.to_string()
                    })).await.ok()
                },
            };
        },

        
        
    }
   
    Ok(())
}