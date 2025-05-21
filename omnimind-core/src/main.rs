// src/main.rs for omnimind-core
use std::io::{self, Write, BufReader, BufRead}; // Added BufReader, BufRead
use std::fs;
use std::path::Path;
use std::fs::File;
use chrono;
use std::process::{Command as OsCommand, Stdio, Child, ChildStdin, ChildStdout}; // For spawning Python
use std::sync::Mutex; // For thread-safe access to Python process handles

// Actix Web and Serde imports
use actix_web::{web, App, HttpServer, Responder, HttpResponse}; // Removed ActixError as it was unused
use actix_cors::Cors;
use serde::{Serialize, Deserialize};
use serde_json;

// --- Structs and Enums ---

// NluResponse: Expected JSON structure from Python NLU script
#[derive(Debug, Serialize, Deserialize)]
struct NluResponse {
    original_text: String,
    intent: String,
    predicted_label: String,
    confidence: f64,
    arguments_text: String,
}

// CommandRequest: Expected JSON from client to our API
#[derive(Debug, Serialize, Deserialize)]
struct CommandRequest {
    raw_command: String,
}

// CommandResponse: Unified JSON structure from our API to client
#[derive(Debug, Serialize)]
struct CommandResponse {
    status: String,
    message: String,
    data: Option<serde_json::Value>,
}

// AppState: Shared state for Actix handlers (Python process handles)
struct AppState {
    py_stdin: Mutex<Option<ChildStdin>>,
    py_stdout_reader: Mutex<Option<BufReader<ChildStdout>>>,
    #[allow(dead_code)] // py_child_process is kept to keep the process alive
    py_child_process: Mutex<Option<Child>>, 
}

// --- Helper: Consistent Help Message ---
const HELP_MESSAGE: &str = "Available commands:\n  echo <text>          - Prints back the text you provide.\n  ls [path]            - Lists files and directories.\n  create_note <title>  - Creates a new text note in the 'omni_notes' directory.\n  ipfs_id              - Fetches the ID of the local IPFS node.\n  ipfs_add <file_path> - Adds a local file to IPFS and returns its CID.\n  ipfs_cat <cid>       - Retrieves and displays content from IPFS for a given CID.\n  help                 - Shows this help message.";


// --- OS Command Implementations (return CommandResponse) ---

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
                    status: "success".to_string(), 
                    message: format!("Created empty note: '{}', but failed to write content: {}", file_path.display(), e),
                    data: Some(serde_json::json!({ "path": file_path.display().to_string() })),
                },
            }
        }
        Err(e) => CommandResponse { status: "error".to_string(), message: format!("Error creating note file '{}': {}", file_path.display(), e), data: None },
    }
}

async fn get_ipfs_id_for_api() -> CommandResponse {
    let ipfs_api_url = "http://127.0.0.1:5001/api/v0/id";
    let client = reqwest::Client::new();
    match client.post(ipfs_api_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<IpfsIdResponse>().await { // IpfsIdResponse needs to be defined
                    Ok(ipfs_id_data) => CommandResponse {
                        status: "success".to_string(),
                        message: "Successfully fetched IPFS Node ID.".to_string(),
                        data: Some(serde_json::json!({ "ipfsNodeId": ipfs_id_data.id })),
                    },
                    Err(e) => CommandResponse { status: "error".to_string(), message: format!("Failed to parse IPFS ID response: {}", e), data: None },
                }
            } else {
                let status_code = response.status();
                let error_text = response.text().await.unwrap_or_else(|e| format!("Unknown error and failed to get error text: {}", e));
                CommandResponse { status: "error".to_string(), message: format!("IPFS API request failed with status {}: {}", status_code, error_text), data: None }
            }
        }
        Err(e) => CommandResponse { status: "error".to_string(), message: format!("Failed to connect to IPFS API: {}. Ensure IPFS daemon is running and API server is enabled at {}.", e, ipfs_api_url), data: None },
    }
}
// We need IpfsIdResponse struct from previous step
#[derive(Debug, Serialize, Deserialize)]
struct IpfsIdResponse {
    #[serde(alias = "ID")]
    id: String,
}


async fn add_file_to_ipfs_for_api(local_file_path_str: &str) -> CommandResponse {
    let file_path = Path::new(local_file_path_str);
    if !file_path.exists() { /* ... error handling ... */ return CommandResponse { status: "error".to_string(), message: format!("Local file '{}' does not exist.", local_file_path_str), data: None }; }
    if !file_path.is_file() { /* ... error handling ... */ return CommandResponse { status: "error".to_string(), message: format!("Path '{}' is not a file.", local_file_path_str), data: None };}

    let file_content_bytes = match fs::read(file_path) { Ok(bytes) => bytes, Err(e) => { return CommandResponse { status: "error".to_string(), message: format!("Failed to read local file '{}': {}", local_file_path_str, e), data: None }; } };
    let file_name = file_path.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let part = match reqwest::multipart::Part::bytes(file_content_bytes).file_name(file_name.clone()).mime_str("application/octet-stream") { Ok(p) => p, Err(e) => { return CommandResponse { status: "error".to_string(), message: format!("Failed to create multipart part: {}", e), data: None }; }};
    let form = reqwest::multipart::Form::new().part("file", part);
    let ipfs_api_url = "http://127.0.0.1:5001/api/v0/add";
    let client = reqwest::Client::new();

    match client.post(ipfs_api_url).multipart(form).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<IpfsAddResponse>().await { // IpfsAddResponse needs to be defined
                    Ok(add_data) => CommandResponse {
                        status: "success".to_string(),
                        message: format!("File '{}' successfully added to IPFS.", add_data.name),
                        data: Some(serde_json::json!({ "fileName": add_data.name, "cid": add_data.hash, "size": add_data.size })),
                    },
                    Err(e) => CommandResponse { status: "error".to_string(), message: format!("Successfully uploaded but failed to parse IPFS add response: {}", e), data: None },
                }
            } else { /* ... error handling for non-success status ... */ 
                let status_code = response.status();
                let error_text = response.text().await.unwrap_or_else(|e| format!("Unknown error: {}", e));
                CommandResponse { status: "error".to_string(), message: format!("IPFS API /add failed with {}: {}", status_code, error_text), data: None }
            }
        }
        Err(e) => CommandResponse { status: "error".to_string(), message: format!("Failed to connect to IPFS API /add: {}", e), data: None },
    }
}
// We need IpfsAddResponse struct from previous step
#[derive(Debug, Serialize, Deserialize)]
struct IpfsAddResponse {
    #[serde(alias = "Name")]
    name: String,
    #[serde(alias = "Hash")]
    hash: String,
    #[serde(alias = "Size")]
    size: String,
}

async fn cat_file_from_ipfs_for_api(cid_str: &str) -> CommandResponse {
    if (!cid_str.starts_with("Qm") && !cid_str.starts_with("ba")) || cid_str.len() < 46 { /* ... error handling ... */ return CommandResponse { status: "error".to_string(), message: format!("Invalid CID format: '{}'", cid_str), data: None }; }
    let ipfs_api_url = format!("http://127.0.0.1:5001/api/v0/cat?arg={}", cid_str);
    let client = reqwest::Client::new();
    match client.post(&ipfs_api_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(content) => CommandResponse {
                        status: "success".to_string(),
                        message: format!("Successfully retrieved content for CID: {}", cid_str),
                        data: Some(serde_json::json!({ "cid": cid_str, "content": content })),
                    },
                    Err(e) => CommandResponse { status: "error".to_string(), message: format!("Failed to read content for CID {}: {}", cid_str, e), data: None },
                }
            } else { /* ... detailed error handling for non-success status ... */ 
                let status_code = response.status();
                let error_body = response.text().await.unwrap_or_else(|e| format!("Failed to read error body: {}", e));
                let mut error_message_from_ipfs = error_body.clone();
                if let Ok(json_error) = serde_json::from_str::<serde_json::Value>(&error_body) {
                    if let Some(msg) = json_error.get("Message").and_then(|v| v.as_str()) { error_message_from_ipfs = msg.to_string(); }
                }
                CommandResponse { status: "error".to_string(), message: format!("IPFS API /cat for {} failed with {}: {}", cid_str, status_code, error_message_from_ipfs), data: None }
            }
        }
        Err(e) => CommandResponse { status: "error".to_string(), message: format!("Failed to connect to IPFS API /cat (CID: {}): {}", cid_str, e), data: None },
    }
}


// --- Main Command Processing Logic (now includes NLU fallback) ---
async fn process_omni_command(
    raw_command_str: &str,
    app_state: web::Data<AppState>, // Pass AppState here
) -> CommandResponse {
    let parts: Vec<&str> = raw_command_str.trim().split_whitespace().collect();
    if parts.is_empty() {
        return CommandResponse { status: "error".to_string(), message: "Empty command.".to_string(), data: None };
    }

    let command_keyword = parts[0].to_lowercase();
    let args_str: Vec<String> = parts.get(1..).unwrap_or(&[]).iter().map(|s| s.to_string()).collect();

    // --- Hybrid Approach: Try keyword match first ---
    let direct_match_result = match command_keyword.as_str() {
        "echo" => Some(CommandResponse { 
            status: "success".to_string(), 
            message: args_str.join(" "), 
            data: None 
        }),
        "help" => Some(CommandResponse { 
            status: "success".to_string(), 
            message: HELP_MESSAGE.to_string(), 
            data: None 
        }),
        "ls" => Some(list_directory_contents_for_api(args_str.get(0).map(String::as_str).unwrap_or("."))),
        "create_note" => {
            if args_str.is_empty() { Some(CommandResponse { status: "error".to_string(), message: "Usage: create_note <title_of_note>".to_string(), data: None }) } 
            else { Some(create_note_for_api(&args_str.join(" "))) }
        },
        "ipfs_id" => Some(get_ipfs_id_for_api().await),
        "ipfs_add" => {
            if args_str.is_empty() { Some(CommandResponse { status: "error".to_string(), message: "Usage: ipfs_add <local_file_path>".to_string(), data: None }) }
            else { Some(add_file_to_ipfs_for_api(&args_str.join(" ")).await) }
        },
        "ipfs_cat" => {
            if args_str.is_empty() { Some(CommandResponse { status: "error".to_string(), message: "Usage: ipfs_cat <ipfs_cid>".to_string(), data: None }) }
            else { Some(cat_file_from_ipfs_for_api(&args_str[0]).await) }
        },
        _ => None, // No direct keyword match
    };

    if let Some(response) = direct_match_result {
        return response; // Return if keyword matched
    }

    // --- If no direct keyword match, try NLU fallback ---
    println!("No direct command match for '{}'. Trying NLU fallback...", raw_command_str.trim());
    
    let py_stdin_opt = app_state.py_stdin.lock();
    let py_stdout_reader_opt = app_state.py_stdout_reader.lock();

    let mut py_stdin_guard = match py_stdin_opt { Ok(g) => g, Err(_) => return CommandResponse { status: "error".to_string(), message: "NLU stdin lock error".to_string(), data: None }};
    let mut py_stdout_reader_guard = match py_stdout_reader_opt { Ok(g) => g, Err(_) => return CommandResponse { status: "error".to_string(), message: "NLU stdout lock error".to_string(), data: None }};

    let py_stdin = match py_stdin_guard.as_mut() { Some(s) => s, None => return CommandResponse { status: "error".to_string(), message: "NLU stdin not available.".to_string(), data: None }};
    let py_stdout_reader = match py_stdout_reader_guard.as_mut() { Some(r) => r, None => return CommandResponse { status: "error".to_string(), message: "NLU stdout not available.".to_string(), data: None }};
    
    if writeln!(py_stdin, "{}", raw_command_str.trim()).is_err() || py_stdin.flush().is_err() {
        eprintln!("Failed to write/flush to Python NLU stdin.");
        return CommandResponse { status: "error".to_string(), message: "Failed to send command to NLU service.".to_string(), data: None };
    }

    let mut nlu_response_json_str = String::new();
    if py_stdout_reader.read_line(&mut nlu_response_json_str).is_err() {
        eprintln!("Failed to read from Python NLU stdout.");
        return CommandResponse { status: "error".to_string(), message: "Failed to receive response from NLU service.".to_string(), data: None };
    }

    match serde_json::from_str::<NluResponse>(nlu_response_json_str.trim()) {
        Ok(nlu_result) => {
            println!("NLU Result: intent='{}', confidence={:.2}, args='{}'", 
                     nlu_result.intent, nlu_result.confidence, nlu_result.arguments_text);
            const NLU_CONFIDENCE_THRESHOLD: f64 = 0.5;
            if nlu_result.confidence >= NLU_CONFIDENCE_THRESHOLD {
                let nlu_command_keyword = nlu_result.intent.as_str();
                let nlu_args_str: Vec<String> = nlu_result.arguments_text.split_whitespace().map(String::from).collect();
                
                            // Re-dispatch based on NLU result
                            // ... inside if nlu_result.confidence >= NLU_CONFIDENCE_THRESHOLD {
                let mut command_execution_response = match nlu_command_keyword {
                    "echo" => CommandResponse { 
                        status: "success".to_string(), 
                        message: nlu_args_str.join(" "), 
                        data: None // Base data
                    },
                    "help" => {
                        CommandResponse { 
                            status: "success".to_string(), 
                            message: HELP_MESSAGE.to_string(), 
                            data: None // Base data
                        }
                    },
                    "ls" => {
                        // Recalculate path_to_list for 'ls' as done before
                        let mut path_to_list_nlu = ".".to_string();
                        let args_text_lower_nlu = nlu_result.arguments_text.to_lowercase();
                        if args_text_lower_nlu.contains("parent directory") { path_to_list_nlu = "..".to_string(); }
                        // ... (add other path logic from previous ls NLU handler) ...
                        else if !nlu_args_str.is_empty() {
                            let first_arg_lower = nlu_args_str[0].to_lowercase();
                            if !["me", "my", "the", "a", "in", "all"].contains(&first_arg_lower.as_str()) {
                                path_to_list_nlu = nlu_args_str[0].clone();
                            }
                        }
                        println!("NLU for 'ls': determined path_to_list = '{}' from args_text = '{}'", 
                                path_to_list_nlu, nlu_result.arguments_text);
                        list_directory_contents_for_api(&path_to_list_nlu)
                    },
                    "create_note" => {
                        if nlu_args_str.is_empty() { CommandResponse { status: "error".to_string(), message: "NLU: create_note requires a title.".to_string(), data: None } }
                        else { create_note_for_api(&nlu_args_str.join(" ")) }
                    },
                    "ipfs_id" => get_ipfs_id_for_api().await,
                    "ipfs_add" => {
                        if nlu_args_str.is_empty() { CommandResponse { status: "error".to_string(), message: "NLU: ipfs_add requires a file path.".to_string(), data: None } }
                        else { add_file_to_ipfs_for_api(&nlu_args_str.join(" ")).await }
                    },
                    "ipfs_cat" => {
                        if nlu_args_str.is_empty() { CommandResponse { status: "error".to_string(), message: "NLU: ipfs_cat requires a CID.".to_string(), data: None } }
                        else { cat_file_from_ipfs_for_api(&nlu_args_str[0]).await }
                    },
                    "quit" => CommandResponse {
                        status: "info".to_string(),
                        message: "NLU suggested 'quit'. Server does not quit via API. Use Ctrl+C on server.".to_string(),
                        data: None,
                    },
                    _ => CommandResponse { 
                        status: "error".to_string(), 
                        message: format!("NLU identified intent '{}', but it's unhandled after NLU processing.", nlu_result.intent),
                        data: None // Base data
                    }
                };

                // Now, augment the response with NLU confidence if it was a success/info
                if command_execution_response.status == "success" || command_execution_response.status == "info" {
                    let nlu_info = serde_json::json!({
                        "nlu_confidence": nlu_result.confidence,
                        "nlu_predicted_label": nlu_result.predicted_label,
                        "nlu_intent_mapped_to": nlu_result.intent
                    });

                    if let Some(existing_data) = command_execution_response.data.take() {
                        // If there was already data, merge NLU info into it.
                        // This assumes existing_data is an object, or we create a new object.
                        if let serde_json::Value::Object(mut map) = existing_data {
                            map.insert("nlu_details".to_string(), nlu_info);
                            command_execution_response.data = Some(serde_json::Value::Object(map));
                        } else {
                            // If existing_data wasn't an object, or we want to keep it separate
                            command_execution_response.data = Some(serde_json::json!({
                                "original_data": existing_data,
                                "nlu_details": nlu_info
                            }));
                        }
                    } else {
                        command_execution_response.data = Some(nlu_info);
                    }
                }
                return command_execution_response; // Return the augmented response
            } else {
                CommandResponse { status: "error".to_string(), message: format!("NLU confidence {:.2} for intent '{}' too low.", nlu_result.confidence, nlu_result.intent), data: Some(serde_json::json!({"nlu_result": nlu_result})) }
            }
        }
        Err(e) => {
            eprintln!("Failed to parse NLU JSON response: {}. Raw: '{}'", e, nlu_response_json_str.trim());
            CommandResponse { status: "error".to_string(), message: "Error processing NLU response.".to_string(), data: None }
        }
    }
}

// --- Actix Web Handler ---
async fn handle_command_request(
    req: web::Json<CommandRequest>,
    app_state: web::Data<AppState>, // Inject AppState
) -> impl Responder {
    let response = process_omni_command(&req.raw_command, app_state).await;
    HttpResponse::Ok().json(response)
}

// --- Main function to start the server ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 OmniMind Core API server starting...");

    let python_executable = "python"; // Or "python3"
    // IMPORTANT: Adjust this path if omnimind-core is not run from its own directory
    // For now, assumes `cargo run` is executed from within `omnimind-core/`
    let nlu_script_path = "../omnimind-nlu-py/nlu_server.py"; 

    println!("Attempting to spawn Python NLU script: {} {}", python_executable, nlu_script_path);

    let mut py_process_cmd = OsCommand::new(python_executable);
    py_process_cmd.arg(nlu_script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit()); // Inherit stderr to see Python logs/errors directly

    // Set current directory for the Python script if needed, so it can find models if using relative paths.
    // For Hugging Face cache, this is usually not needed as it uses user's home dir.
    // py_process_cmd.current_dir("../omnimind-nlu-py/"); // Example if script needs specific CWD

    let mut py_process = match py_process_cmd.spawn() {
        Ok(child) => {
            println!("✅ Python NLU script spawned successfully (PID: {}).", child.id());
            child
        }
        Err(e) => {
            eprintln!("❌ Failed to spawn Python NLU script: {}", e);
            eprintln!("Ensure Python is installed ('{}') and accessible via PATH.", python_executable);
            eprintln!("Ensure the script exists at relative path: '{}' from where omnimind-core is run.", nlu_script_path);
            std::process::exit(1); 
        }
    };

    let py_stdin = py_process.stdin.take().expect("Failed to open Python stdin pipe");
    let py_stdout = py_process.stdout.take().expect("Failed to open Python stdout pipe");
    let py_stdout_reader = BufReader::new(py_stdout);
    
    let app_state = web::Data::new(AppState {
        py_stdin: Mutex::new(Some(py_stdin)),
        py_stdout_reader: Mutex::new(Some(py_stdout_reader)),
        py_child_process: Mutex::new(Some(py_process)),
    });

    let server_address = "127.0.0.1:3030";
    println!("🎧 OmniMind Core API server listening on http://{}", server_address);
    println!("Send POST requests to /command with JSON: {{ \"raw_command\": \"your command here\" }}");

    HttpServer::new(move || {
        let cors = Cors::default()
              .allowed_origin("http://localhost:3000") 
              .allowed_methods(vec!["GET", "POST"])   
              .allowed_headers(vec![actix_web::http::header::AUTHORIZATION, actix_web::http::header::ACCEPT, actix_web::http::header::CONTENT_TYPE])
              .max_age(3600);

        App::new()
            .app_data(app_state.clone()) 
            .wrap(cors)
            .route("/command", web::post().to(handle_command_request))
    })
    .bind(server_address)?
    .run()
    .await
    // Graceful shutdown of Python script would go here if we implement it.
    // For example, by taking py_child_process from app_state and sending "__EXIT__" or killing it.
}