# OmniMind NLU Python Service (omnimind-nlu-py)

This directory contains the Python-based Natural Language Understanding (NLU) service for OmniMind.

## Overview

nlu_server.py is a script that uses the Hugging Face transformers library to provide NLU capabilities (specifically, zero-shot intent classification) to the omnimind-core Rust application.

It is designed to be spawned as a child process by omnimind-core. Communication happens via:
- omnimind-core sends raw text commands to this script's stdin.
- This script processes the text, performs NLU, and prints a JSON response to its stdout.
- Log messages from this script are printed to stderr and are typically inherited by the parent omnimind-core process.

## Current Functionality

- Intent Classification: Uses a pre-trained zero-shot classification model (currently facebook/bart-large-mnli) to map user input text to one of several predefined candidate command intents (e.g., "list items", "create note").
- Output: Returns a JSON object containing:
    - original_text: The input text.
    - intent: The canonical command keyword (e.g., "ls", "create_note") mapped from the predicted label.
    - predicted_label: The raw label predicted by the model.
    - confidence: The model's confidence score for the prediction.
    - arguments_text: A simple extraction of text assumed to be arguments (currently everything after the first word of the input).
- Control: Can be terminated gracefully by sending __EXIT__ via its stdin.

## Technology Stack

- Language: Python
- Machine Learning Library: Hugging Face transformers
- ML Backend: PyTorch (torch)

## Getting Started & Running (Standalone Test)

1.  Ensure Python 3.7+ and pip are installed.
2.  Navigate to the omnimind-nlu-py directory.
3.  Install dependencies:
        pip install transformers torch
    
4.  Run the script directly for testing:
        python nlu_server.py
    
5.  The script will initialize (downloading the model on first run if not cached) and then wait for input from stdin. Type commands and press Enter to see JSON output. Type __EXIT__ to terminate.

Note: This script is primarily intended to be run as a child process by omnimind-core, not as a standalone server in production.

Refer to the main project README.md in the repository root for full setup instructions for all OmniMind components.