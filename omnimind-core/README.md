# OmniMind Core (omnimind-core)

This directory contains the Rust-based core component of the OmniMind operating system.

## Overview

omnimind-core serves as the backend engine for OmniMind. It is responsible for:
- Managing OS-level tasks (file system operations, etc.).
- Processing user commands received via its local HTTP API.
- Interacting with other local services (like an IPFS daemon).
- Orchestrating Natural Language Understanding (NLU) by communicating with the omnimind-nlu-py Python script via IPC.
- Eventually, managing P2P network interactions and modular plugins.

## Current Functionality (via API at http://localhost:3030/command)

- Command Processing: Accepts JSON POST requests with a raw_command field.
- Hybrid NLU:
    - Attempts direct keyword matching for known commands.
    - If no match, falls back to the Python NLU service for intent classification.
    - Executes commands based on NLU intent if confidence is sufficient.
- Implemented Commands:
    - help: Displays available commands.
    - echo <text>: Echoes back the provided text.
    - ls [path]: Lists files and directories.
    - create_note <title>: Creates a new text note in ./omni_notes/.
    - ipfs_id: Fetches the ID of the local IPFS node.
    - ipfs_add <file_path>: Adds a local file to IPFS.
    - ipfs_cat <cid>: Retrieves file content from IPFS by CID.
- API Server: Runs an Actix web server (default: http://127.0.0.1:3030) with CORS enabled for local development with omnimind-client (from http://localhost:3000).

## Technology Stack

- Language: Rust
- Web Framework: Actix-web
- Serialization: Serde (for JSON)
- HTTP Client: Reqwest (for IPFS API calls)
- Concurrency: Tokio (via Actix and Reqwest)
- Timestamping: Chrono

## Getting Started & Running

1.  Ensure Rust and Cargo are installed.
2.  Navigate to the omnimind-core directory.
3.  Build the project:
        cargo build
    
4.  Run the server:
        cargo run
    
5.  The server will start, and if the NLU integration is active, it will attempt to spawn the ../omnimind-nlu-py/nlu_server.py script. Ensure Python and its dependencies (transformers, torch) are set up in that directory.
6.  Ensure an IPFS daemon (like IPFS Desktop) is running for IPFS commands to function.

Refer to the main project README.md in the repository root for full setup instructions for all OmniMind components.