require("@nomicfoundation/hardhat-toolbox");
require("dotenv").config(); // This line loads environment variables from .env

/** @type import('hardhat/config').HardhatUserConfig */

const alchemySepoliaApiUrl = process.env.ALCHEMY_SEPOLIA_API_URL;
const sepoliaPrivateKey = process.env.SEPOLIA_PRIVATE_KEY;

module.exports = {
  solidity: "0.8.20", // Ensure this matches your contract's pragma
  networks: {
    hardhat: {
      // Configuration for the local Hardhat Network (used by default for tests, etc.)
    },
    sepolia: {
      url: alchemySepoliaApiUrl || "", // Fallback to empty string if not set
      accounts: sepoliaPrivateKey ? [sepoliaPrivateKey] : [], // Fallback to empty array
      // gasPrice: 20000000000, // Optional: 20 Gwei, adjust if needed
    },
  },
  etherscan: {
    // To verify your contract on Etherscan (optional, for later)
    // apiKey: process.env.ETHERSCAN_API_KEY // Get an API key from https://etherscan.io/
  },
};