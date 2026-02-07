// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{Manager, State};
use tokio::sync::mpsc;
use base64::Engine;

// =========================
// App-wide constants
// =========================
const API_BASE: &str = "https://flowen.eu"; // change to http://localhost:3001 when testing locally
const UPLOAD_PATH: &str = "/api/upload?skipEmail=true";
const TEAMS_PATH: &str = "/api/teams";
const LOGIN_PATH: &str = "/api/auth/login";
const REFRESH_PATH: &str = "/api/auth/refresh"; // optional — only works if backend exposes it

// How long to wait after each processed upload in the queue (prevents hammering)
const QUEUE_UPLOAD_DELAY_MS: u64 = 100;
// Minimal debounce window for file-change bursts
const DEBOUNCE_SECONDS: u64 = 3;
// If access token expires within this window, attempt refresh before uploading
const TOKEN_REFRESH_SAFETY_WINDOW_SECS: i64 = 30;

// =========================
// STATE MANAGEMENT
// =========================
#[derive(Debug)]
struct AppState {
  // Auth
  jwt_token: Mutex<Option<String>>,      // access token (Bearer)
  refresh_token: Mutex<Option<String>>,  // optional: in case API returns it in JSON

  // Selection / watcher status
  selected_team_id: Mutex<Option<String>>, // currently selected team
  is_watching: Mutex<bool>,

  // Debounce + in-flight tracking
  upload_timestamps: Mutex<HashMap<String, u64>>, // last-upload unix secs per path
  inflight_paths: Mutex<HashSet<String>>,         // paths currently queued/processing

  // Upload queue
  upload_tx: Mutex<Option<mpsc::Sender<(String, String)>>>, // (path, teamId)

  // Shared HTTP client with cookie store enabled (for refresh cookie if backend sets it)
  http: reqwest::Client,
}

impl AppState {
  fn new() -> Self {
    let http = reqwest::ClientBuilder::new()
      .cookie_store(true)
      .build()
      .expect("failed to build reqwest client");

    Self {
      jwt_token: Mutex::new(None),
      refresh_token: Mutex::new(None),
      selected_team_id: Mutex::new(None),
      is_watching: Mutex::new(false),
      upload_timestamps: Mutex::new(HashMap::new()),
      inflight_paths: Mutex::new(HashSet::new()),
      upload_tx: Mutex::new(None),
      http,
    }
  }
}

// =========================
// DATA STRUCTURES
// =========================
#[derive(Serialize, Deserialize)]
struct LoginRequest {
  email: String,
  password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginResponse {
  token: String,
  #[serde(default)]
  refresh_token: Option<String>, // optional JSON field
}

// =========================
// UTILITIES
// =========================
fn now_unix_secs() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs()
}

fn get_default_sync_folder() -> String {
  #[cfg(target_os = "windows")]
  {
    // NOTE: fix the username if needed — or resolve dynamically via dirs crate
    "C:\\Users\\DanielOIsson\\Flowen".to_string()
  }
  #[cfg(not(target_os = "windows"))]
  {
    format!(
      "{}/Flowen",
      std::env::var("HOME").unwrap_or("/home/user".to_string())
    )
  }
}

fn get_mime_type(file_path: &str) -> String {
  let path = std::path::Path::new(file_path);
  let mime = if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
    match &ext.to_lowercase()[..] {
      // Documents
      "pdf" => "application/pdf",
      "doc" => "application/msword",
      "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      "xls" => "application/vnd.ms-excel",
      "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      "ppt" => "application/vnd.ms-powerpoint",
      "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
      // Images
      "jpg" | "jpeg" => "image/jpeg",
      "png" => "image/png",
      "gif" => "image/gif",
      "webp" => "image/webp",
      "svg" => "image/svg+xml",
      // Video
      "mp4" => "video/mp4",
      "webm" => "video/webm",
      "mov" => "video/quicktime",
      // Text
      "txt" => "text/plain",
      "html" | "htm" => "text/html",
      "css" => "text/css",
      "js" => "text/javascript",
      "json" => "application/json",
      _ => "application/octet-stream",
    }
  } else {
    "application/octet-stream"
  };
  mime.to_string()
}

async fn read_file(path: &str) -> Result<Vec<u8>, String> {
  std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))
}

fn decode_jwt_exp(token: &str) -> Option<i64> {
  // Very light-weight decoder: split token, base64 decode payload, parse exp
  let payload_b64 = token.split('.').nth(1)?;
  let mut padded = payload_b64.to_string();
  while padded.len() % 4 != 0 {
    padded.push('=');
  }
  let decoded = base64::engine::general_purpose::STANDARD.decode(padded).ok()?;
  let payload_json = String::from_utf8(decoded).ok()?;
  let v: serde_json::Value = serde_json::from_str(&payload_json).ok()?;
  v.get("exp")?.as_i64()
}

fn token_expiring_soon(token: &str) -> bool {
  if let Some(exp) = decode_jwt_exp(token) {
    let now = chrono::Utc::now().timestamp();
    exp - now <= TOKEN_REFRESH_SAFETY_WINDOW_SECS
  } else {
    true // if we cannot decode, assume it needs refresh
  }
}

async fn try_refresh_access_token(state: &AppState) -> Result<String, String> {
  // Requires backend to expose REFRESH_PATH and to set an httpOnly refresh cookie at login.
  // We use the shared client with cookie_store=true so cookies persist.
  let url = format!("{}{}", API_BASE, REFRESH_PATH);
  let resp = state
    .http
    .post(url)
    .send()
    .await
    .map_err(|e| format!("Refresh request failed: {}", e))?;

  if !resp.status().is_success() {
    return Err(format!("Refresh denied ({}).", resp.status()));
  }

  let text = resp
    .text()
    .await
    .map_err(|e| format!("Failed to read refresh response: {}", e))?;
  let v: serde_json::Value = serde_json::from_str(&text)
    .map_err(|e| format!("Failed to parse refresh JSON: {}", e))?;
  let new_token = v
    .get("token")
    .and_then(|t| t.as_str())
    .ok_or("No token in refresh response")?;

  // store
  if let Ok(mut guard) = state.jwt_token.lock() {
    *guard = Some(new_token.to_string());
  }

  Ok(new_token.to_string())
}

async fn ensure_access_token(state: &AppState) -> Result<String, String> {
  let mut need_refresh = false;
  let current_token = {
    let guard = state.jwt_token.lock().unwrap();
    guard.clone().ok_or("Not logged in. Call login_to_flowen first.")?
  };

  if token_expiring_soon(&current_token) {
    need_refresh = true;
  }

  if need_refresh {
    match try_refresh_access_token(state).await {
      Ok(t) => return Ok(t),
      Err(e) => {
        // If refresh is not available, fall back to existing token and let server decide.
        eprintln!("⚠️ Token refresh failed: {} — proceeding with existing token", e);
      }
    }
  }

  Ok(current_token)
}

// =========================
// TAURI COMMANDS
// =========================
#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_default_sync_folder_cmd() -> String { get_default_sync_folder() }

#[tauri::command]
fn create_sync_folder(path: String) -> Result<String, String> {
  std::fs::create_dir_all(&path)
    .map(|_| format!("Folder created: {}", path))
    .map_err(|e| format!("Failed to create folder: {}", e))
}

#[tauri::command]
fn mount_as_drive(folder_path: String, drive_letter: String) -> Result<String, String> {
  #[cfg(target_os = "windows")]
  {
    let output = std::process::Command::new("subst")
      .arg(format!("{}:", drive_letter))
      .arg(&folder_path)
      .output();

    match output {
      Ok(result) if result.status.success() => {
        Ok(format!("Mounted {} as {}:", folder_path, drive_letter))
      }
      Ok(result) => Err(format!(
        "Failed to mount drive: {}",
        String::from_utf8_lossy(&result.stderr)
      )),
      Err(e) => Err(format!("Command failed: {}", e)),
    }
  }
  #[cfg(not(target_os = "windows"))]
  {
    Err("Drive mounting only supported on Windows".to_string())
  }
}

#[tauri::command]
fn stop_watching(state: State<'_, AppState>) -> Result<String, String> {
  let mut is_watching = state.is_watching.lock().unwrap();
  *is_watching = false;
  println!("🛑 Stopped watching");
  Ok("Stopped watching".to_string())
}

#[tauri::command]
fn set_selected_team(team_id: String, state: State<'_, AppState>) -> Result<String, String> {
  let mut selected_team = state.selected_team_id.lock().unwrap();
  *selected_team = Some(team_id.clone());
  println!("✅ Selected team set to: {}", team_id);
  Ok(format!("Selected team: {}", team_id))
}

#[tauri::command]
async fn get_user_teams(state: State<'_, AppState>) -> Result<String, String> {
  let token_str = ensure_access_token(&state).await?;
  let url = format!("{}{}", API_BASE, TEAMS_PATH);

  let response = state
    .http
    .get(url)
    .header("Authorization", format!("Bearer {}", token_str))
    .send()
    .await
    .map_err(|e| format!("Request failed: {}", e))?;

  let text = response
    .text()
    .await
    .map_err(|e| format!("Failed to read response: {}", e))?;

  println!("🏢 Teams: {}", text);
  Ok(text)
}

#[tauri::command]
async fn login_to_flowen(email: String, password: String, state: State<'_, AppState>) -> Result<String, String> {
  println!("🔐 Logging in to Flowen...\n   Email: {}", email);

  let url = format!("{}{}", API_BASE, LOGIN_PATH);
  let login_data = serde_json::json!({ "email": email, "password": password });

  let resp = state
    .http
    .post(url)
    .json(&login_data)
    .send()
    .await
    .map_err(|e| format!("Request failed: {}", e))?;

  let status = resp.status();
  let body = resp
    .text()
    .await
    .map_err(|e| format!("Failed to read response: {}", e))?;

  if !status.is_success() {
    return Err(format!("Login failed ({}): {}", status, body));
  }

  let parsed: LoginResponse = serde_json::from_str(&body)
    .map_err(|e| format!("Failed to parse JSON: {}", e))?;

  {
    let mut t = state.jwt_token.lock().unwrap();
    *t = Some(parsed.token.clone());
  }
  if let Some(rt) = parsed.refresh_token {
    let mut r = state.refresh_token.lock().unwrap();
    *r = Some(rt);
  }

  // Debug: print payload
  if let Some(payload_part) = parsed.token.split('.').nth(1) {
    let mut padded = payload_part.to_string();
    while padded.len() % 4 != 0 { padded.push('='); }
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(padded) {
      if let Ok(json_str) = String::from_utf8(decoded) {
        println!("🔍 JWT Payload: {}", json_str);
      }
    }
  }

  println!("✅ Logged in successfully!\n   Token(first20): {}", &parsed.token.chars().take(20).collect::<String>());
  Ok("Logged in successfully".to_string())
}

// Core uploader usable by both queue worker and command
async fn upload_file_to_flowen_core(
  state: &AppState,
  file_path: String,
  team_id: String,
) -> Result<String, String> {
  println!("📤 Starting file upload to Flowen...\n   File: {}\n   Team ID: {}", file_path, team_id);

  // Ensure token (refresh if needed)
  let token_str = ensure_access_token(state).await?;

  // Read file
  let data = read_file(&file_path).await?;
  let file_len = data.len();
  println!("✅ File read: {} bytes", file_len);
  println!("📤 Uploading file without client-side encryption (server will encrypt)");

  // relative path
  let sync_folder = get_default_sync_folder();
  let file_path_obj = std::path::Path::new(&file_path);
  let sync_folder_obj = std::path::Path::new(&sync_folder);

  let relative_path = file_path_obj
    .strip_prefix(&sync_folder_obj)
    .map_err(|_| "File is not in sync folder")?;
  let relative_path_str = relative_path
    .to_str()
    .ok_or("Invalid path encoding")?
    .replace('\\', "/");

  let file_name = file_path_obj
    .file_name()
    .and_then(|n| n.to_str())
    .ok_or("Invalid filename")?;

  let mime_type = get_mime_type(&file_path);
  println!("📋 MIME type detected: {}", mime_type);

  // Build multipart
  let form = reqwest::multipart::Form::new()
    .part(
      "files",
      reqwest::multipart::Part::bytes(data)
        .file_name(file_name.to_string())
        .mime_str(&mime_type)
        .map_err(|e| format!("Failed to set mime type: {}", e))?,
    )
    .text("senderEmail", "flowen-sync@industrinat.se")
    .text("senderName", "Flowen Desktop Sync")
    .text("receiverEmail", "daniel.olsson@industrinat.se")
    .text("teamId", team_id.clone())
    .text("relativePath", relative_path_str.clone());

  let url = format!("{}{}", API_BASE, UPLOAD_PATH);
  println!("📦 Uploading to {}", url);

  let resp = state
    .http
    .post(url)
    .header("Authorization", format!("Bearer {}", token_str))
    .multipart(form)
    .send()
    .await
    .map_err(|e| format!("Upload request failed: {}", e))?;

  let status = resp.status();
  let text = resp
    .text()
    .await
    .map_err(|e| format!("Failed to read response: {}", e))?;

  if status.is_success() {
    println!("✅ Upload successful!\n   Response: {}", text);
    Ok(format!(
      "✅ Uploaded: {} ({} bytes)",
      relative_path_str, file_len
    ))
  } else {
    Err(format!("Upload failed ({}): {}", status, text))
  }
}

#[tauri::command]
async fn upload_file_to_flowen(
  file_path: String,
  team_id: String,
  state: State<'_, AppState>,
) -> Result<String, String> {
  let res = upload_file_to_flowen_core(&state, file_path.clone(), team_id.clone()).await;

  // On success, update debounce map
  if res.is_ok() {
    let current = now_unix_secs();
    if let Ok(mut ts) = state.upload_timestamps.lock() {
      ts.insert(file_path.clone(), current);
    }
  }

  res
}

#[tauri::command]
async fn initial_sync(
  folder_path: String,
  state: State<'_, AppState>,
) -> Result<String, String> {
  use std::fs;
  use std::path::Path;

  println!("🔄 Starting initial sync...\n   Folder: {}", folder_path);

  let team_id = {
    let team = state.selected_team_id.lock().unwrap();
    team.as_ref().ok_or("No team selected")?.clone()
  };
  println!("✅ Team: {}", team_id);

  // Scan files
  let mut files_to_upload: Vec<String> = Vec::new();
  fn scan_directory(dir: &Path, files: &mut Vec<String>) -> std::io::Result<()> {
    if dir.is_dir() {
      for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
          scan_directory(&path, files)?;
        } else if path.is_file() {
          if let Some(s) = path.to_str() { files.push(s.to_string()); }
        }
      }
    }
    Ok(())
  }
  scan_directory(Path::new(&folder_path), &mut files_to_upload)
    .map_err(|e| format!("Failed to scan directory: {}", e))?;

  println!("📂 Found {} local files", files_to_upload.len());
  if files_to_upload.is_empty() { return Ok("No files to sync".to_string()); }

  // Enqueue to queue (sequential processing in the worker)
  let mut uploaded = 0usize;
  let mut failed = 0usize;
  for p in files_to_upload {
    // Guard against duplicates already in queue
    let should_enqueue = {
      let mut inflight = state.inflight_paths.lock().unwrap();
      if inflight.contains(&p) {
        false
      } else {
        inflight.insert(p.clone());
        true
      }
    };
    if !should_enqueue { continue; }

    let tx_opt = state.upload_tx.lock().unwrap().clone();
    if let Some(tx) = tx_opt {
      if tx.try_send((p.clone(), team_id.clone())).is_ok() {
        uploaded += 1; // counting scheduled, actual results appear in logs
      } else {
        failed += 1;
      }
    } else {
      failed += 1;
    }
  }

  let msg = format!(
    "Initial sync queued: {} files scheduled, {} failed to schedule",
    uploaded, failed
  );
  println!("✅ {}", msg);
  Ok(msg)
}

#[tauri::command]
async fn start_watching(
  folder_path: String,
  state: State<'_, AppState>,
  app_handle: tauri::AppHandle,
) -> Result<String, String> {
  use notify::{recommended_watcher, EventKind, RecursiveMode, Watcher};
  use std::sync::mpsc::channel;

  let selected_team = {
    let team = state.selected_team_id.lock().unwrap();
    team.as_ref()
      .ok_or("No team selected. Please select a team first.")?
      .clone()
  };

println!("📁 Starting file watcher for team: {}", selected_team);
  {
    let mut w = state.is_watching.lock().unwrap();
    *w = true;
  }
  println!("📁 Watching folder: {}", folder_path);

  let (tx_events, rx_events) = channel();
  
  let selected_team_clone = selected_team.clone();
  let app_handle_clone = app_handle.clone();

  let mut watcher = recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
    match res {
      Ok(event) => {
        if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
          return;
        }
        for path in &event.paths {
          if let Some(path_str) = path.to_str() {
            println!("📁 File changed: {}", path_str);
            let path_string = path_str.to_string();
            let handle = app_handle_clone.clone();
            let team = selected_team_clone.clone();

            tauri::async_runtime::spawn(async move {
              let state: State<AppState> = handle.state();

              // Debounce check
              let now = now_unix_secs();
              let should_upload = {
                let mut timestamps = state.upload_timestamps.lock().unwrap();
                if let Some(&last) = timestamps.get(&path_string) {
                  let diff = now.saturating_sub(last);
                  if diff < DEBOUNCE_SECONDS {
                    println!("⏭️ Skipping (debounce) — {}s ago", diff);
                    false
                  } else {
                    timestamps.insert(path_string.clone(), now);
                    true
                  }
                } else {
                  timestamps.insert(path_string.clone(), now);
                  true
                }
              };
              if !should_upload { return; }

              // Duplicate check
              let ok_to_queue = {
                let mut inflight = state.inflight_paths.lock().unwrap();
                if inflight.contains(&path_string) {
                  false
                } else {
                  inflight.insert(path_string.clone());
                  true
                }
              };
              if !ok_to_queue { return; }

              // Enqueue
              let tx_opt = {
                let guard = state.upload_tx.lock().unwrap();
                guard.clone()
              };
              if let Some(tx) = tx_opt {
                if let Err(e) = tx.try_send((path_string.clone(), team)) {
                  println!("❌ Failed to enqueue: {}", e);
                  state.inflight_paths.lock().unwrap().remove(&path_string);
                }
              }
            });
          }
        }
        let _ = tx_events.send(event);
      }
      Err(e) => println!("❌ Watch error: {:?}", e),
    }
  })
  .map_err(|e| format!("Failed to create watcher: {}", e))?;

  watcher
    .watch(std::path::Path::new(&folder_path), RecursiveMode::Recursive)
    .map_err(|e| format!("Failed to watch folder: {}", e))?;

  println!("✅ Watcher started for team: {}", selected_team);

  std::thread::spawn(move || {
    let _watcher = watcher;
    println!("🔔 File watcher thread started...");
    for event in rx_events {
      println!("📢 Event: {:?}", event.kind);
    }
    println!("⚠️ Watcher thread stopped!");
  });

  Ok(format!("Started watching: {} for team: {}", folder_path, selected_team))
}

// =========================
// MAIN (Tauri builder)
// =========================
fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_store::Builder::default().build())
    .plugin(tauri_plugin_autostart::init(
      tauri_plugin_autostart::MacosLauncher::LaunchAgent,
      Some(vec!["--flag1", "--flag2"]),
    ))
    .manage(AppState::new())
   .setup(|app| {
      // Create upload queue worker
      let state: State<AppState> = app.state();
      let (tx, mut rx) = mpsc::channel::<(String, String)>(200);

      // Save sender
      {
        let mut guard = state.upload_tx.lock().unwrap();
        *guard = Some(tx.clone());
      }

      let app_handle = app.handle().clone(); // FIX: Clone the handle
      tauri::async_runtime::spawn(async move {
        loop {
          tokio::select! {
            biased;
            maybe = rx.recv() => {
              if let Some((path, team_id)) = maybe {
                let state: State<AppState> = app_handle.state();
                match upload_file_to_flowen_core(&state, path.clone(), team_id.clone()).await {
                  Ok(msg) => println!("✅ {}", msg),
                  Err(err) => println!("❌ Upload failed in queue: {}", err),
                }
                // Remove inflight mark
                state.inflight_paths.lock().unwrap().remove(&path);
                // Small delay between uploads
                tokio::time::sleep(Duration::from_millis(QUEUE_UPLOAD_DELAY_MS)).await;
              } else {
                // channel closed
                tokio::time::sleep(Duration::from_millis(250)).await;
              }
            }
          }
        }
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      greet,
      get_default_sync_folder_cmd,
      create_sync_folder,
      mount_as_drive,
      start_watching,
      stop_watching,
      login_to_flowen,
      upload_file_to_flowen,
      get_user_teams,
      set_selected_team,
      initial_sync,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
