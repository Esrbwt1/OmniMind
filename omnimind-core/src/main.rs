// src/main.rs for omnimind-core
use std::io::{self, Write}; // Import necessary modules for I/O

fn main() {
    println!("Welcome to OmniMind Core!");
    println!("--------------------------");
    println!("Type 'help' for a list of commands, or 'quit' to exit.");

    loop {
        print!("omni> "); // Print the command prompt
        io::stdout().flush().unwrap(); // Ensure the prompt is displayed before reading input

        let mut input = String::new(); // Create a mutable string to store user input

        io::stdin()
            .read_line(&mut input) // Read a line from standard input
            .expect("Failed to read line");

        let trimmed_input = input.trim(); // Remove leading/trailing whitespace

        if trimmed_input.is_empty() {
            continue; // If input is empty, show prompt again
        }

        // Simple command parsing: split by whitespace
        // The first part is the command, the rest are arguments
        let mut parts = trimmed_input.split_whitespace();
        let command = parts.next().unwrap_or(""); // Get the command, or empty if no input
        let args: Vec<&str> = parts.collect(); // Collect the rest as arguments

        match command.to_lowercase().as_str() {
            "echo" => {
                if args.is_empty() {
                    println!("Usage: echo <text to echo>");
                } else {
                    println!("{}", args.join(" ")); // Join arguments and print
                }
            }
            "help" => {
                println!("Available commands:");
                println!("  echo <text>  - Prints back the text you provide.");
                println!("  help         - Shows this help message.");
                println!("  quit         - Exits OmniMind Core.");
                // Add more commands here later
            }
            "quit" | "exit" => {
                println!("Exiting OmniMind Core. Goodbye!");
                break; // Exit the loop
            }
            "" => {
                // Empty command after trimming (should have been caught by is_empty, but good fallback)
            }
            _ => {
                println!("Unknown command: '{}'. Type 'help' for available commands.", command);
            }
        }
    }
}