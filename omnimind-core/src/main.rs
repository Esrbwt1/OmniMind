// src/main.rs for omnimind-core
use std::io::{self, Write}; // Keep for potential direct output if needed
use std::fs;
use std::path::Path;
use std::fs::File;
use chrono;
use actix_cors::Cors;

// Actix Web and Serde imports
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Error as ActixError};
use serde::{Serialize, Deserialize};

// --- Command Definition and Parsing Logic (from previous steps, slightly adapted) ---
#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone, Serialize, Deserialize
enum CommandType { // Renamed from Command to avoid conflict with a potential Command struct
    Echo,
    Help,
    Quit, // Will be handled differently for server
    ListFiles,
    CreateNote,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandRequest {
    raw_command: String,
}

// Define a unified response structure
#[derive(Debug, Serialize)]
struct CommandResponse {
    status: String,        // "success" or "error"
    message: String,       // User-friendly message or error detail
    data: Option<serde_json::Value>, // Optional structured data (e.g., file list)
}

// This function will now process the command and return a CommandResponse
// It's adapted to be called by the web handler.
fn process_omni_command(raw_command_str: &str) -> CommandResponse {
    let parts: Vec<&str> = raw_command_str.trim().split_whitespace().collect();
    if parts.is_empty() {
        return CommandResponse {
            status: "error".to_string(),
            message: "Empty command received.".to_string(),
            data: None,
        };
    }

    let command_keyword = parts[0].to_lowercase();
    let args_str: Vec<String> = parts.get(1..)
        .unwrap_or(&[])
        .iter()
        .map(|s| s.to_string())
        .collect();

    match command_keyword.as_str() {
        "echo" => {
            if args_str.is_empty() {
                CommandResponse {
                    status: "error".to_string(),
                    message: "Usage: echo <text to echo>".to_string(),
                    data: None,
                }
            } else {
                CommandResponse {
                    status: "success".to_string(),
                    message: args_str.join(" "),
                    data: None,
                }
            }
        }
        "help" => {
            let help_text = [
                "Available commands:",
                "  echo <text>          - Prints back the text you provide.",
                "  ls [path]            - Lists files and directories.",
                "  create_note <title>  - Creates a new text note in the 'omni_notes' directory.",
                "  help                 - Shows this help message.",
                // "quit" / "exit" are not useful for a server endpoint this way
            ].join("\n");
            CommandResponse {
                status: "success".to_string(),
                message: help_text,
                data: None,
            }
        }
        "ls" => {
            let path_to_list = args_str.get(0).map(String::as_str).unwrap_or(".");
            list_directory_contents_for_api(path_to_list)
        }
        "create_note" => {
            if args_str.is_empty() {
                CommandResponse {
                    status: "error".to_string(),
                    message: "Usage: create_note <title_of_note>".to_string(),
                    data: None,
                }
            } else {
                let title = args_str.join(" ");
                create_note_for_api(&title)
            }
        }
        _ => CommandResponse {
            status: "error".to_string(),
            message: format!("Unknown command: '{}'", command_keyword),
            data: None,
        },
    }
}

// --- OS Command Implementations (adapted for API response) ---

fn list_directory_contents_for_api(dir_path_str: &str) -> CommandResponse {
    let path = Path::new(dir_path_str);
    if !path.exists() {
        return CommandResponse { status: "error".to_string(), message: format!("Path '{}' does not exist.", dir_path_str), data: None };
    }
    if !path.is_dir() {
        return CommandResponse { status: "error".to_string(), message: format!("Path '{}' is not a directory.", dir_path_str), data: None };
    }

    let mut entries_vec = Vec::new();
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry_result in entries {
                if let Ok(entry) = entry_result {
                    let entry_path = entry.path();
                    let file_name_str = entry.file_name().to_string_lossy().into_owned();
                    let entry_type = if entry_path.is_dir() { "Directory" } else if entry_path.is_file() { "File" } else { "Other" };
                    entries_vec.push(serde_json::json!({ "name": file_name_str, "type": entry_type }));
                }
            }
            CommandResponse {
                status: "success".to_string(),
                message: format!("Contents of '{}':", dir_path_str),
                data: Some(serde_json::Value::Array(entries_vec)),
            }
        }
        Err(e) => CommandResponse { status: "error".to_string(), message: format!("Error reading directory '{}': {}", dir_path_str, e), data: None },
    }
}

fn create_note_for_api(title: &str) -> CommandResponse {
    let notes_dir = Path::new("./omni_notes");
    if !notes_dir.exists() {
        if let Err(e) = fs::create_dir(notes_dir) {
            return CommandResponse { status: "error".to_string(), message: format!("Error creating notes directory './omni_notes/': {}", e), data: None };
        }
    }

    let sane_title = title.chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_').collect::<String>().replace(" ", "_");
    if sane_title.is_empty() {
        return CommandResponse { status: "error".to_string(), message: "Note title invalid after sanitization.".to_string(), data: None };
    }

    let file_name = format!("{}.omni.txt", sane_title);
    let file_path = notes_dir.join(&file_name);

    match File::create(&file_path) {
        Ok(mut file) => {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let content = format!("OmniMind Note\nTitle: {}\nCreated: {}\n\n---\n(Start your note here)\n", title, timestamp);
            match file.write_all(content.as_bytes()) {
                Ok(_) => CommandResponse {
                    status: "success".to_string(),
                    message: format!("Successfully created note: '{}'", file_path.display()),
                    data: Some(serde_json::json!({ "path": file_path.display().to_string() })),
                },
                Err(e) => CommandResponse {
                    status: "success".to_string(), // File created, but content write failed
                    message: format!("Created empty note: '{}', but failed to write content: {}", file_path.display(), e),
                    data: Some(serde_json::json!({ "path": file_path.display().to_string() })),
                },
            }
        }
        Err(e) => CommandResponse { status: "error".to_string(), message: format!("Error creating note file '{}': {}", file_path.display(), e), data: None },
    }
}

// --- Actix Web Handler ---
async fn handle_command_request(req: web::Json<CommandRequest>) -> impl Responder {
    // Log the received command (optional)
    // println!("Received command via API: {}", req.raw_command);
    
    let response = process_omni_command(&req.raw_command);
    HttpResponse::Ok().json(response) // Always return Ok for the HTTP response itself; actual success/error is in JSON
}

// --- Main function to start the server ---
#[actix_web::main] // Macro to setup Actix runtime
async fn main() -> std::io::Result<()> {
    let server_address = "127.0.0.1:3030"; // OmniMind Core API server
    println!("🚀 OmniMind Core API server starting on http://{}", server_address);
    println!("Send POST requests to /command with JSON: {{ \"raw_command\": \"your command here\" }}");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000") // Allow your React app's origin
            .allowed_methods(vec!["GET", "POST"])    // Allow POST and GET (if you add GET later)
            .allowed_headers(vec![actix_web::http::header::AUTHORIZATION, actix_web::http::header::ACCEPT, actix_web::http::header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(cors) // Add CORS middleware
            .route("/command", web::post().to(handle_command_request))
    })
    .bind(server_address)?
    .run()
    .await
}

// Note: The old interactive CLI loop is removed for this server version.
// It could be added back as an optional mode or a separate binary later.