// src/main.rs for omnimind-core
use std::io::{self, Write}; // Write is already there, good.
use std::fs;
use std::path::Path;
use std::fs::File;
use chrono;

// Define an enum for our commands
#[derive(Debug)] // Allow printing the enum for debugging
enum Command {
    Echo(Vec<String>),      // Command "echo" with its arguments
    Help,                   // Command "help"
    Quit,                   // Command "quit" or "exit"
    ListFiles(Option<String>), // Command "ls" with an optional path argument
    CreateNote(String),    // Command "create_note" with the note title
    Unknown(String),        // For any unrecognized command
}

// Function to parse the raw input string into a Command enum
fn parse_input(input: &str) -> Command {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Command::Unknown("".to_string()); // Or a specific NoOp command
    }

    let command_str = parts[0].to_lowercase();
    let args = parts.get(1..).unwrap_or(&[]).iter().map(|s| s.to_string()).collect::<Vec<String>>();

    match command_str.as_str() {
        "echo" => Command::Echo(args),
        "help" => Command::Help,
        "quit" | "exit" => Command::Quit,
        "ls" => {
            // For "ls", an optional argument is the path
            Command::ListFiles(args.get(0).cloned())
        }
        "create_note" => {
            if args.is_empty() {
                println!("Usage: create_note <title_of_note>");
                Command::Unknown("create_note: missing title".to_string())
            } else {
                Command::CreateNote(args.join(" ")) // Join all args to form the title
            }
        }
        _ => Command::Unknown(command_str),
    }
}

fn list_directory_contents(dir_path_str: &str) {
    let path = Path::new(dir_path_str);

    if !path.exists() {
        println!("Error: Path '{}' does not exist.", dir_path_str);
        return;
    }

    if !path.is_dir() {
        println!("Error: Path '{}' is not a directory.", dir_path_str);
        // Optionally, if it's a file, just print its name or details
        // For now, we only list contents of directories for 'ls'
        return;
    }

    println!("Contents of '{}':", dir_path_str);
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry_result in entries {
                match entry_result {
                    Ok(entry) => {
                        let entry_path = entry.path();
                        let file_name = entry.file_name();
                        let file_name_str = file_name.to_string_lossy(); // Convert OsString to String

                        if entry_path.is_dir() {
                            println!("  {}/ (Directory)", file_name_str);
                        } else if entry_path.is_file() {
                            // You could add file size or other metadata here later
                            // let metadata = fs::metadata(&entry_path).ok();
                            // let size = metadata.map_or(0, |m| m.len());
                            // println!("  {} (File, {} bytes)", file_name_str, size);
                            println!("  {} (File)", file_name_str);
                        } else {
                            println!("  {} (Other)", file_name_str); // Symlinks, etc.
                        }
                    }
                    Err(e) => println!("  Error reading entry: {}", e),
                }
            }
        }
        Err(e) => {
            println!("Error reading directory contents: {}", e);
        }
    }
}

fn main() {
    println!("Welcome to OmniMind Core! (Rule-Based NLU v0.1)");
    println!("-------------------------------------------------");
    println!("Type 'help' for a list of commands, or 'quit' to exit.");

    loop {
        print!("omni> ");
        io::stdout().flush().unwrap();

        let mut user_input_str = String::new();
        io::stdin().read_line(&mut user_input_str).expect("Failed to read line");

        if user_input_str.trim().is_empty() {
            continue;
        }

        let command = parse_input(&user_input_str);

        // println!("DEBUG: Parsed command: {:?}", command); // Uncomment for debugging

        match command {
            Command::Echo(args_vec) => {
                if args_vec.is_empty() {
                    println!("Usage: echo <text to echo>");
                } else {
                    println!("{}", args_vec.join(" "));
                }
            }
            Command::Help => {
                println!("Available commands:");
                println!("  echo <text>          - Prints back the text you provide.");
                println!("  ls [path]            - Lists files and directories.");
                println!("  create_note <title>  - Creates a new text note in the 'omni_notes' directory.");
                println!("  help                 - Shows this help message.");
                println!("  quit / exit          - Exits OmniMind Core.");
            }
            Command::Quit => {
                println!("Exiting OmniMind Core. Goodbye!");
                break;
            }
            Command::ListFiles(path_opt) => {
                match path_opt {
                    Some(path_str) => {
                        // Call our new function here
                        list_directory_contents(&path_str);
                    }
                    None => {
                        // Get current directory and call our new function
                        match std::env::current_dir() {
                            Ok(current_path_buf) => {
                                match current_path_buf.to_str() {
                                    Some(current_path_str) => list_directory_contents(current_path_str),
                                    None => println!("Error: Could not convert current path to string."),
                                }
                            }
                            Err(e) => println!("Error getting current directory: {}", e),
                        }
                    }
                }
            }
            Command::CreateNote(title) => {
                // Let's create a notes directory if it doesn't exist in the current path
                let notes_dir = Path::new("./omni_notes");
                if !notes_dir.exists() {
                    match fs::create_dir(notes_dir) {
                        Ok(_) => println!("Created notes directory: ./omni_notes/"),
                        Err(e) => {
                            println!("Error creating notes directory './omni_notes/': {}", e);
                            // Optionally, we could decide to not proceed if dir creation fails
                            // For now, we'll let it try to create the file anyway if dir creation failed
                            // but the file path construction will still use it.
                        }
                    }
                }

                // Sanitize title to be a safe filename (simple sanitization for now)
                let sane_title = title
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                    .collect::<String>()
                    .replace(" ", "_"); // Replace spaces with underscores

                if sane_title.is_empty() {
                    println!("Error: Note title is invalid after sanitization (became empty).");
                } else {
                    let file_name = format!("{}.omni.txt", sane_title);
                    let file_path = notes_dir.join(&file_name); // Path: ./omni_notes/Sane_Title.omni.txt

                    println!("Attempting to create note: '{}' at '{}'", title, file_path.display());

                    match File::create(&file_path) {
                        Ok(mut file) => {
                            // Optionally write some placeholder content
                            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                            let content = format!(
                                "OmniMind Note\nTitle: {}\nCreated: {}\n\n---\n(Start your note here)\n",
                                title, timestamp
                            );
                            match file.write_all(content.as_bytes()) {
                                Ok(_) => {
                                    println!("Successfully created note: '{}'", file_path.display());
                                    println!("Original title: '{}'", title);
                                }
                                Err(e) => {
                                    println!("Successfully created empty note: '{}', but failed to write content: {}", file_path.display(), e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error creating note file '{}': {}", file_path.display(), e);
                        }
                    }
                }
            }
            Command::Unknown(cmd_str) => {
                if !cmd_str.is_empty() { // Avoid printing for empty unrecognized commands
                    println!("Unknown command: '{}'. Type 'help' for available commands.", cmd_str);
                }
            }
        }
    }
}