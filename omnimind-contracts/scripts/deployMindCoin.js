// We require the Hardhat Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
//
// You can also run a script with `npx hardhat run <script>`. If you do that, Hardhat
// will compile your contracts, add the Hardhat Runtime Environment's members to the
// global scope, and execute the script.
const hre = require("hardhat");

async function main() {
  // Get the deployer's address
  const [deployer] = await hre.ethers.getSigners();
  console.log("Deploying contracts with the account:", deployer.address);

  const initialOwner = deployer.address; // The deployer will be the initial owner

  // Get the ContractFactory for MindCoin
  const MindCoin = await hre.ethers.getContractFactory("MindCoin");

  // Deploy the MindCoin contract, passing the initialOwner to the constructor
  console.log("Deploying MindCoin...");
  const mindCoin = await MindCoin.deploy(initialOwner);

  // Wait for the contract to be deployed
  await mindCoin.waitForDeployment();

  // Log the address of the deployed MindCoin contract
  // Note: In Hardhat Ethers v6 and later, the contract address is on mindCoin.target
  // For older versions it might be mindCoin.address
  // We will use mindCoin.target for compatibility with newer versions.
  const deployedAddress = mindCoin.target || mindCoin.address; // Fallback for older versions
  console.log("MindCoin deployed to:", deployedAddress);

  // You can add more logic here, like verifying the contract on Etherscan
  // or interacting with it post-deployment.

  // Example: Log the deployer's MindCoin balance
  const balance = await mindCoin.balanceOf(deployer.address);
  console.log(`Deployer's MindCoin balance: ${hre.ethers.formatUnits(balance, 18)} MIND`);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});