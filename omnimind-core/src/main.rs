// src/main.rs for omnimind-core
use std::io::{self, Write}; // Keep for potential direct output if needed
use std::fs;
use std::path::Path;
use std::fs::File;
use chrono;
use actix_cors::Cors;
use reqwest::multipart;

// Actix Web and Serde imports
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
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

#[derive(Debug, Serialize, Deserialize)]
struct IpfsIdResponse {
    #[serde(alias = "ID")] // Allow deserializing from "ID"
    id: String,
    // We can add other fields like PublicKey if needed later
}

#[derive(Debug, Serialize, Deserialize)]
struct IpfsAddResponse {
    #[serde(alias = "Name")]
    name: String,
    #[serde(alias = "Hash")]
    hash: String, // This is the CID
    #[serde(alias = "Size")]
    size: String,
}

// This function will now process the command and return a CommandResponse
// It's adapted to be called by the web handler.
async fn process_omni_command(raw_command_str: &str) -> CommandResponse {
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
                "  ipfs_id              - Fetches the ID of the local IPFS node.", // NEW
                "  ipfs_add <file_path> - Adds a local file to IPFS and returns its CID.",
                "  ipfs_cat <cid>       - Retrieves and displays content from IPFS for a given CID.",
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
        "ipfs_id" => { // NOW WE CAN CALL THE ASYNC FUNCTION
            get_ipfs_id_for_api().await // Await the async call
        }
        "ipfs_add" => { 
            if args_str.is_empty() {
                CommandResponse {
                    status: "error".to_string(),
                    message: "Usage: ipfs_add <local_file_path>".to_string(),
                    data: None,
                }
            } else {
                // For now, assume the first argument is the full path.
                // In a real OS, you'd handle relative paths, tilde expansion etc.
                let file_path_to_add = args_str.join(" "); // If path has spaces
                add_file_to_ipfs_for_api(&file_path_to_add).await
            }
        }
        "ipfs_cat" => {
            if args_str.is_empty() {
                CommandResponse {
                    status: "error".to_string(),
                    message: "Usage: ipfs_cat <ipfs_cid>".to_string(),
                    data: None,
                }
            } else {
                let cid_to_cat = &args_str[0]; // Assume CID is the first argument
                cat_file_from_ipfs_for_api(cid_to_cat).await
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

async fn get_ipfs_id_for_api() -> CommandResponse {
    let ipfs_api_url = "http://127.0.0.1:5001/api/v0/id"; // Default IPFS daemon API URL for id

    // Ensure your IPFS Desktop daemon is running and API is accessible at this address.
    // You can test in browser: http://127.0.0.1:5001/api/v0/id (might show error if not from allowed origin, but daemon should log access attempt)
    // Or with curl: curl -X POST http://127.0.0.1:5001/api/v0/id

    let client = reqwest::Client::new();
    match client.post(ipfs_api_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<IpfsIdResponse>().await {
                    Ok(ipfs_id_data) => CommandResponse {
                        status: "success".to_string(),
                        message: "Successfully fetched IPFS Node ID.".to_string(),
                        data: Some(serde_json::json!({ "ipfsNodeId": ipfs_id_data.id })),
                    },
                    Err(e) => CommandResponse {
                        status: "error".to_string(),
                        message: format!("Failed to parse IPFS ID response: {}", e),
                        data: None,
                    },
                }
            } else {
                let status_code = response.status(); // Get status BEFORE consuming response
                let error_text = response.text().await.unwrap_or_else(|e| format!("Unknown error and failed to get error text: {}", e));
                CommandResponse {
                    status: "error".to_string(),
                    message: format!("IPFS API request failed with status {}: {}", status_code, error_text), // Use status_code
                    data: None,
                }
            }
        }
        Err(e) => CommandResponse {
            status: "error".to_string(),
            message: format!("Failed to connect to IPFS API: {}. Ensure IPFS daemon is running and API server is enabled at {}.", e, ipfs_api_url),
            data: None,
        },
    }
}

async fn add_file_to_ipfs_for_api(local_file_path_str: &str) -> CommandResponse {
    let file_path = Path::new(local_file_path_str);

    if !file_path.exists() {
        return CommandResponse { status: "error".to_string(), message: format!("Local file '{}' does not exist.", local_file_path_str), data: None };
    }
    if !file_path.is_file() {
        return CommandResponse { status: "error".to_string(), message: format!("Path '{}' is not a file.", local_file_path_str), data: None };
    }

    // Read file content. For large files, streaming would be better.
    // For MVP, reading into memory is simpler.
    let file_content_bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            return CommandResponse { status: "error".to_string(), message: format!("Failed to read local file '{}': {}", local_file_path_str, e), data: None };
        }
    };

    let file_name = file_path.file_name().unwrap_or_default().to_string_lossy().into_owned();

    // Create a multipart part for the file
    let part = match multipart::Part::bytes(file_content_bytes)
        .file_name(file_name.clone()) // Set the filename for the part
        .mime_str("application/octet-stream") 
    {
        Ok(p) => p,
        Err(e) => {
             return CommandResponse { status: "error".to_string(), message: format!("Failed to create multipart part for file: {}", e), data: None };
        }
    };


    // Create the multipart form
    let form = multipart::Form::new().part("file", part); // The field name "file" is expected by IPFS /add

    let ipfs_api_url = "http://127.0.0.1:5001/api/v0/add";
    let client = reqwest::Client::new();

    match client.post(ipfs_api_url)
        .multipart(form)
        .send()
        .await 
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<IpfsAddResponse>().await {
                    Ok(add_data) => CommandResponse {
                        status: "success".to_string(),
                        message: format!("File '{}' successfully added to IPFS.", add_data.name),
                        data: Some(serde_json::json!({
                            "fileName": add_data.name,
                            "cid": add_data.hash,
                            "size": add_data.size
                        })),
                    },
                    Err(e) => CommandResponse {
                        status: "error".to_string(),
                        message: format!("Successfully uploaded but failed to parse IPFS add response: {}", e),
                        data: None, // You could include raw response text here if helpful
                    },
                }
            } else {
                let status_code = response.status();
                let error_text = response.text().await.unwrap_or_else(|e| format!("Unknown error and failed to get error text: {}", e));
                CommandResponse {
                    status: "error".to_string(),
                    message: format!("IPFS API /add request failed with status {}: {}", status_code, error_text),
                    data: None,
                }
            }
        }
        Err(e) => CommandResponse {
            status: "error".to_string(),
            message: format!("Failed to connect to IPFS API for /add: {}. Ensure IPFS daemon is running.", e),
            data: None,
        },
    }
}


async fn cat_file_from_ipfs_for_api(cid_str: &str) -> CommandResponse {
    // Basic CID validation (very simple, not exhaustive)
    if !cid_str.starts_with("Qm") && !cid_str.starts_with("ba") { // Common v0 and v1 prefixes
        return CommandResponse { status: "error".to_string(), message: format!("Invalid CID format: '{}'. CIDs usually start with 'Qm' (v0) or 'ba' (v1).", cid_str), data: None };
    }
    if cid_str.len() < 46 { // Typical CID length
         return CommandResponse { status: "error".to_string(), message: format!("CID '{}' seems too short.", cid_str), data: None };
    }


    let ipfs_api_url = format!("http://127.0.0.1:5001/api/v0/cat?arg={}", cid_str);
    let client = reqwest::Client::new();

    match client.post(&ipfs_api_url) // IPFS /cat is a POST endpoint expecting the CID as an arg
        .send()
        .await 
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await { // Get the content as text
                    Ok(content) => CommandResponse {
                        status: "success".to_string(),
                        message: format!("Successfully retrieved content for CID: {}", cid_str),
                        data: Some(serde_json::json!({ "cid": cid_str, "content": content })),
                    },
                    Err(e) => CommandResponse {
                        status: "error".to_string(),
                        message: format!("Successfully connected but failed to read content for CID {}: {}", cid_str, e),
                        data: None,
                    },
                }
            } else {
                let status_code = response.status();
                // Try to get error text from IPFS, which might be plain text or JSON
                let error_body = response.text().await.unwrap_or_else(|e| format!("Failed to read error body: {}", e));
                let mut error_message_from_ipfs = error_body.clone();

                // IPFS errors are sometimes JSON like {"Message": "...", "Code": 0, "Type": "error"}
                if let Ok(json_error) = serde_json::from_str::<serde_json::Value>(&error_body) {
                    if let Some(msg) = json_error.get("Message").and_then(|v| v.as_str()) {
                        error_message_from_ipfs = msg.to_string();
                    }
                }
                
                CommandResponse {
                    status: "error".to_string(),
                    message: format!("IPFS API /cat request for CID {} failed with status {}: {}", cid_str, status_code, error_message_from_ipfs),
                    data: None,
                }
            }
        }
        Err(e) => CommandResponse {
            status: "error".to_string(),
            message: format!("Failed to connect to IPFS API for /cat (CID: {}): {}. Ensure IPFS daemon is running.", cid_str, e),
            data: None,
        },
    }
}

// --- Actix Web Handler ---
async fn handle_command_request(req: web::Json<CommandRequest>) -> impl Responder {
    // Log the received command (optional)
    // println!("Received command via API: {}", req.raw_command);
    
    let response = process_omni_command(&req.raw_command).await; // Add .await here
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