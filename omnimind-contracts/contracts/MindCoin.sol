// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract MindCoin is ERC20, Ownable {
    constructor(address initialOwner) ERC20("MindCoin", "MIND") Ownable(initialOwner) {
        _mint(initialOwner, 1000000000 * 10**decimals()); // 1 billion tokens
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }

    // Function to reward contributors - can only be called by the owner
    // This is a simple placeholder; a more robust system might involve proposals or multisig
    function rewardContributor(address contributor, uint256 rewardAmount) public onlyOwner {
        _mint(contributor, rewardAmount);
        // Optionally, emit an event here
        // emit ContributionRewarded(contributor, rewardAmount);
    }
}