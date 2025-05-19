// src/main.rs for omnimind-core
use std::io::{self, Write};
use std::fs; // For file system operations
use std::path::Path; // For working with file paths

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
                println!("  ls [path]            - Lists files (placeholder). Optional path.");
                println!("  create_note <title>  - Creates a new note (placeholder).");
                println!("  help                 - Shows this help message.");
                println!("  quit / exit          - Exits OmniMind Core.");
            }
            Command::Quit => {
                println!("Exiting OmniMind Core. Goodbye!");
                break;
            }
            // ... inside the main loop's match command { ... }
            Command::ListFiles(path_opt) => {
                match path_opt {
                    Some(path_str) => {
                        println!("Placeholder: Listing files in directory '{}'...", path_str);
                        // Call our new function here
                        list_directory_contents(&path_str);
                    }
                    None => {
                        println!("Placeholder: Listing files in current directory...");
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
                println!("Placeholder: Creating note with title '{}'...", title);
                // Future: Implement actual note creation logic here
                println!("Note '{}' created successfully (placeholder).", title);
            }
            Command::Unknown(cmd_str) => {
                if !cmd_str.is_empty() { // Avoid printing for empty unrecognized commands
                    println!("Unknown command: '{}'. Type 'help' for available commands.", cmd_str);
                }
            }
        }
    }
}