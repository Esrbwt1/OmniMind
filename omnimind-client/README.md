# OmniMind Client (omnimind-client)

This directory contains the React-based web frontend for the OmniMind operating system.

## Overview

omnimind-client provides a user interface for interacting with both blockchain-based components (like MindCoin) and the local omnimind-core API server.

## Current Functionality

- Wallet Interaction (MetaMask):
    - Connects to the user's MetaMask wallet.
    - Displays the connected account address.
    - Fetches and displays the user's MindCoin balance from the Sepolia testnet.
    - Allows users to transfer MindCoin to other addresses on Sepolia.
- OmniMind Core Interaction (via local API at `http://localhost:3030`):
    - Provides a general command input field to send commands to omnimind-core.
    - Provides dedicated UI controls for IPFS operations (ipfs_add <path>, ipfs_cat <cid>) that target omnimind-core.
    - Displays structured JSON responses from omnimind-core, including status, messages, data, and NLU confidence details when applicable.
- Deployment: The client is deployable as a static site and is currently hosted on Vercel (though core interaction features require omnimind-core to be running locally).

## Technology Stack

- Framework: React (via Create React App)
- Language: JavaScript (with JSX)
- Ethereum Interaction: ethers.js
- Styling: Basic CSS (default Create React App styles + minimal inline styles)

## Getting Started & Running

1.  Ensure Node.js and npm (or yarn) are installed.
2.  Navigate to the omnimind-client directory.
3.  Install dependencies:
        npm install
    
4.  Start the development server:
        # For Windows (if you encountered OpenSSL errors previously):
    # Ensure your start script in package.json is: 
    # "start": "set NODE_OPTIONS=--openssl-legacy-provider && react-scripts start",
    npm start
    
5.  The client will run on http://localhost:3000 and attempt to connect to omnimind-core API at http://localhost:3030.
6.  Ensure MetaMask is installed in your browser for blockchain features.

Refer to the main project README.md in the repository root for full setup instructions for all OmniMind components.