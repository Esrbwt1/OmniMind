# OmniMind: The AI-Native, Decentralized Operating System

Vision: To build the foundational software for the AI-driven future, empowering users with data sovereignty, seamless human-AI interaction, and a collaborative, decentralized ecosystem.

Live Demo (Client Placeholder with Wallet Interaction): [https://omni-mind-git-main-esrbwt1s-projects.vercel.app/](https://omni-mind-git-main-esrbwt1s-projects.vercel.app/)
*(Connect your MetaMask to the Sepolia testnet to see MindCoin balance functionality.)*

MindCoin (MIND) on Sepolia Testnet: [0x902E7647D1BFc46c4bd741adF488659871240208](https://sepolia.etherscan.io/token/0x902E7647D1BFc46c4bd741adF488659871240208)

---

## About OmniMind

OmniMind is an ambitious open-source project to create a new kind of operating system designed from the ground up for a world where Artificial Intelligence is ubiquitous. It aims to replace traditional OSes (like Windows, macOS, Linux, Android) with a lightweight, modular, and decentralized platform that puts AI at its core.

Key Goals & Principles:
*   AI-Native: Deeply integrated AI for natural language interaction, task automation, and intelligent assistance.
*   Decentralized: Peer-to-peer (P2P) architecture for resource sharing (compute, storage), resilience, and censorship resistance. No central points of failure or control.
*   Data Sovereignty: Users own and control their data, with strong encryption and local-first processing.
*   Open & Collaborative: Fully open-source, with a tokenized economy (MindCoin) to incentivize contributions and participation.
*   Universal & Lightweight: Designed to run on a wide range of devices, from laptops and phones to IoT devices, with a minimal footprint.
*   Modular: A core OS with a plug-and-play module system, allowing the community to extend functionality.

## Current Status (Early MVP - Alpha)

OmniMind is in its very early stages of development. The current codebase represents an initial Minimum Viable Product (MVP) scaffold, demonstrating foundational components:

*   `omnimind-client` (React Frontend):
    *   A web-based client (placeholder UI).
    *   Connects to user's MetaMask wallet.
    *   Displays MindCoin balance from the Sepolia testnet.
    *   Deployed live via Vercel.
*   `omnimind-contracts` (Solidity Smart Contracts):
    *   MindCoin (MIND): An ERC20 token deployed on the Sepolia testnet, serving as the utility and governance token for the ecosystem.
    *   Developed using Hardhat.
*   `omnimind-core` (Rust OS Core):
    *   A basic command-line placeholder.
    *   Compiles and runs, outlining the future direction for OS services.
    *   (AI integration and core OS functionalities are future work).

This project was initiated with the constraint of no capital investment, relying solely on free tools, a single developer's laptop, and a stable internet connection, with an AI assistant (the "Brain & Virtual Brawn") designing the system and providing step-by-step instructions to a human executor (the "Physical Brawn").

## Project Structure

The repository is organized as a monorepo (though not formally managed by monorepo tools yet) with the following main components:

*   omnimind-client/: React-based frontend application.
*   omnimind-contracts/: Solidity smart contracts and Hardhat development environment.
*   omnimind-core/: Rust-based core OS components.

## Getting Started (Development Setup)

To set up the OmniMind development environment locally:

1.  Prerequisites:
    *   [Git](https://git-scm.com/)
    *   [Node.js](https://nodejs.org/) (LTS version, e.g., v18.x or v20.x)
    *   [Rust](https://www.rust-lang.org/tools/install) (including rustc and cargo)
    *   A code editor like [VS Code](https://code.visualstudio.com/) with extensions for Rust (rust-analyzer) and JavaScript/React.
    *   [MetaMask Browser Extension](https://metamask.io/) (for interacting with the client and contracts).
    *   [IPFS Desktop](https://ipfs.io/desktop/) (for future P2P features - optional for current client functionality).

2.  Clone the Repository:
        git clone https://github.com/Esrbwt1/OmniMind.git
    cd OmniMind
    

3.  Set up `omnimind-contracts`:
        cd omnimind-contracts
    npm install
    # To compile contracts:
    # npx hardhat compile
    # To deploy to a local Hardhat node (for testing):
    # npx hardhat node (in one terminal)
    # npx hardhat run scripts/deployMindCoin.js --network localhost (in another terminal)
    
    *Note: To deploy to Sepolia, you'll need a .env file with ALCHEMY_SEPOLIA_API_URL and SEPOLIA_PRIVATE_KEY, and Sepolia ETH in your wallet. Refer to hardhat.config.js.*

4.  Set up `omnimind-client`:
        cd omnimind-client 
    # If you encountered OpenSSL errors previously, ensure your start script in package.json is:
    # For Windows: "start": "set NODE_OPTIONS=--openssl-legacy-provider && react-scripts start",
    # For Linux/macOS: "start": "NODE_OPTIONS=--openssl-legacy-provider react-scripts start",
    npm install
    npm start 
    
    *The client will run on http://localhost:3000.*
    *It requires src/contractInfo.js to be populated with the deployed MindCoin address and ABI (current Sepolia details are in the committed version).*

5.  Set up `omnimind-core`:
        cd omnimind-core
    cargo build
    cargo run
    

## Roadmap (High-Level)

*   Phase 1: Foundational MVP (Current)
    *   [x] Basic project structure (Client, Contracts, Core).
    *   [x] MindCoin ERC20 token deployed on Sepolia.
    *   [x] Client connects to MetaMask and displays MindCoin balance.
    *   [x] Client deployed to Vercel.
    *   [ ] Next: Implement MindCoin transfer in client.
    *   [ ] Next: Basic documentation (this README).
*   Phase 2: Core AI & OS Primitives
    *   [ ] Basic command parser in omnimind-core (Rust).
    *   [ ] Integrate a lightweight local NLU model (e.g., DistilBERT) for command understanding.
    *   [ ] Simple OS task automation via natural language commands (e.g., file ops, app launching).
    *   [ ] Client UI for interacting with omnimind-core (via local API/IPC).
*   Phase 3: Decentralization & P2P
    *   [ ] Integrate IPFS for basic decentralized file storage.
    *   [ ] P2P communication layer (e.g., libp2p) for node discovery and interaction.
    *   [ ] Tokenomics refinement: Staking MindCoin for P2P resource sharing (compute/storage).
*   Phase 4: Advanced AI & Ecosystem Growth
    *   [ ] Integrate more capable (but still lightweight/quantized) generative AI models (e.g., TinyLlama, Phi-3-mini) for simple content/script generation.
    *   [ ] Modular plugin system for community-built modules and AI tools.
    *   [ ] Governance mechanisms using MindCoin.
    *   [ ] Developer tooling and SDKs.

## Contributing

OmniMind is an open-source project, and contributions are highly welcome as the project matures! 
Currently, the best way to contribute is to:
1.  Set up the project locally and explore the codebase.
2.  Provide feedback, report bugs, or suggest features by opening an Issue on GitHub.
3.  Stay tuned for more specific contribution guidelines as core functionalities are developed.

As the project is being bootstrapped with very specific step-by-step execution, direct code contributions might be coordinated. Please express interest in the Issues section.

## License

This project will be licensed under the MIT License (or similar permissive open-source license - to be formally added).

---

*This README is a living document and will be updated as the project progresses.*