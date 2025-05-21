# omnimind-nlu-py/nlu_server.py
import sys
import json
from transformers import pipeline

# Load a zero-shot classification model once when the script starts.
# This might download the model on first run (can be a few hundred MB).
# facebook/bart-large-mnli is a popular choice for this.
# For a potentially smaller/faster model, you could try:
# 'morit/autonlp-zeroshot-topic-detection-Banking77-1658355921' (though may be domain specific)
# 'valhalla/distilbart-mnli-12-3' (smaller distilbart version)
# Let's start with a distilled version for speed and size.
classifier = None
MODEL_NAME = "facebook/bart-large-mnli" 
# Alternative: "facebook/bart-large-mnli" (larger, potentially more accurate)

def initialize_classifier():
    global classifier
    try:
        print(f"NLU_SERVER_LOG: Initializing zero-shot classification pipeline with model: {MODEL_NAME}...", file=sys.stderr)
        classifier = pipeline("zero-shot-classification", model=MODEL_NAME)
        print("NLU_SERVER_LOG: Classifier initialized successfully.", file=sys.stderr)
    except Exception as e:
        print(f"NLU_SERVER_LOG: Error initializing classifier: {e}", file=sys.stderr)
        # Exit if classifier fails to load, as the script is useless without it.
        sys.exit(1)

# Define the candidate labels (our command intents) - REVISED
candidate_labels = [
    "display directory contents", # For 'ls'
    "generate a new text note",   # For 'create_note'
    "repeat user input",          # For 'echo'
    "show ipfs peer identity",    # For 'ipfs_id'
    "upload file to ipfs",        # For 'ipfs_add'
    "download file from ipfs",    # For 'ipfs_cat'
    "show command instructions",  # For 'help'
    "terminate application"       # For 'quit' / 'exit'
]

# Mapping from predicted label to a canonical command keyword for Rust - REVISED
label_to_command_keyword = {
    "display directory contents": "ls",
    "generate a new text note": "create_note",
    "repeat user input": "echo",
    "show ipfs peer identity": "ipfs_id",
    "upload file to ipfs": "ipfs_add",
    "download file from ipfs": "ipfs_cat",
    "show command instructions": "help",
    "terminate application": "quit"
}

def process_command(text_input):
    if not classifier:
        return {"error": "Classifier not initialized."}
    
    try:
        # The sequence is the user's input text
        # The candidate_labels are what we want to classify it against
        # multi_label=False because we expect one primary intent per command
        result = classifier(text_input, candidate_labels, multi_label=False)
        
        # result looks like:
        # {'sequence': '...', 'labels': ['create new note', 'list items', ...], 'scores': [0.99, 0.002, ...]}
        
        predicted_label = result['labels'][0]
        confidence_score = result['scores'][0]
        
        # Map the predicted label to our canonical command keyword
        command_keyword = label_to_command_keyword.get(predicted_label, "unknown")

        # For now, we just return the intent and original text.
        # Later, we can add Named Entity Recognition (NER) to extract arguments.
        return {
            "original_text": text_input,
            "intent": command_keyword, # Our canonical command keyword
            "predicted_label": predicted_label, # The actual label from the model
            "confidence": confidence_score,
            "arguments_text": " ".join(text_input.split()[1:]) # Simple: take everything after first word as args
        }
    except Exception as e:
        print(f"NLU_SERVER_LOG: Error processing command '{text_input}': {e}", file=sys.stderr)
        return {"error": str(e), "original_text": text_input}

if __name__ == "__main__":
    initialize_classifier()
    
    print("NLU_SERVER_LOG: Python NLU Server Ready. Waiting for input...", file=sys.stderr)
    sys.stderr.flush()

    for line in sys.stdin:
        input_text = line.strip()
        
        if input_text.lower() == "__exit__": # Check for EXIT immediately
            print("NLU_SERVER_LOG: Received __EXIT__ command. Shutting down.", file=sys.stderr)
            sys.stderr.flush()
            break # Exit the loop

        if not input_text: # Skip empty lines after checking for exit
            continue

        nlu_result = process_command(input_text)
        print(json.dumps(nlu_result))
        sys.stdout.flush()