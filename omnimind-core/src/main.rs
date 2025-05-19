// src/main.rs for omnimind-core
use std::io::{self, Write};

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
            Command::ListFiles(path_opt) => {
                match path_opt {
                    Some(path) => println!("Placeholder: Listing files in directory '{}'...", path),
                    None => println!("Placeholder: Listing files in current directory..."),
                }
                // Future: Implement actual file listing logic here
                println!("  - file1.txt");
                println!("  - my_document.omni");
                println!("  - sub_folder/");
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