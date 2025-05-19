/* global BigInt */
import React, { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import './App.css';
import { mindCoinAddress, mindCoinABI } from './contractInfo';

function App() {
  const [userAccount, setUserAccount] = useState(null);
  const [mindCoinBalance, setMindCoinBalance] = useState("0");
  const [provider, setProvider] = useState(null);
  // eslint-disable-next-line no-unused-vars
  const [signer, setSigner] = useState(null);
  const [mindCoinContract, setMindCoinContract] = useState(null);
  const [feedbackMessage, setFeedbackMessage] = useState("");


// Effect to initialize provider and potentially signer if wallet is already connected
  useEffect(() => {
    const initProvider = async () => {
      if (window.ethereum) {
        const newProvider = new ethers.BrowserProvider(window.ethereum);
        setProvider(newProvider);

        try {
          const signers = await newProvider.listAccounts();
          if (signers.length > 0) {
            // listAccounts returns Signer objects in ethers v6
            const initialSigner = signers[0];
            const initialAccountAddress = await initialSigner.getAddress();
            await handleAccountsChanged([initialAccountAddress], newProvider, initialSigner); // Pass string address
          }
        } catch (err) {
          console.warn("Could not automatically connect to existing accounts:", err);
          // This can happen if the user hasn't connected before or has disconnected
        }


        window.ethereum.on('accountsChanged', (accounts) => handleAccountsChanged(accounts, newProvider)); // newProvider will be in closure
        window.ethereum.on('chainChanged', handleChainChanged);

        return () => {
          if (window.ethereum.removeListener) { // Check if removeListener exists
              window.ethereum.removeListener('accountsChanged', (accounts) => handleAccountsChanged(accounts, newProvider));
              window.ethereum.removeListener('chainChanged', handleChainChanged);
          }
        };
      } else {
        setFeedbackMessage("MetaMask (or another Ethereum wallet) is not installed. Please install it to use this app.");
      }
    };
    initProvider();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Empty dependency array means this runs once on mount


  // Naming convention: Pass the provider explicitly if it might not be set in state yet
  const handleAccountsChanged = async (accounts, currentProvider, preResolvedSigner = null) => {
    const localProvider = currentProvider || provider; // Use passed provider or state provider

    if (!localProvider) {
        setFeedbackMessage("Provider not available. Please ensure MetaMask is connected.");
        return;
    }

    if (accounts.length === 0) {
      setFeedbackMessage("Wallet disconnected. Please connect your wallet.");
      setUserAccount(null);
      setSigner(null); // We set signer in state, so disable the eslint warning
      setMindCoinContract(null);
      setMindCoinBalance("0");
    } else {
      const accountAddress = accounts[0]; // This should now be a string from eth_requestAccounts or getAddress()
      
      if (typeof accountAddress !== 'string') {
          console.error("Received non-string account address:", accountAddress);
          setFeedbackMessage("Error: Received invalid account format.");
          return;
      }
      
      setUserAccount(accountAddress);
      setFeedbackMessage(`Wallet connected: ${accountAddress.substring(0, 6)}...${accountAddress.substring(accountAddress.length - 4)}`);
      
      let currentSigner = preResolvedSigner;
      if (!currentSigner) {
          currentSigner = await localProvider.getSigner();
      }
      setSigner(currentSigner); // We set signer in state, so disable the eslint warning

      const contract = new ethers.Contract(mindCoinAddress, mindCoinABI, currentSigner);
      setMindCoinContract(contract);
    }
  };

  const handleChainChanged = (_chainId) => {
    setFeedbackMessage("Network changed. Reloading...");
    window.location.reload();
  };


  useEffect(() => {
    const fetchBalance = async () => {
      if (mindCoinContract && userAccount) {
        try {
          // Ensure we are on Sepolia (Chain ID 11155111)
          const network = await provider.getNetwork();
          if (network.chainId !== BigInt(11155111)) { // Note: chainId is a BigInt
            setFeedbackMessage("Please connect to the Sepolia test network in MetaMask.");
            setMindCoinBalance("N/A (Wrong Network)");
            return;
          }

          setFeedbackMessage("Fetching MindCoin balance...");
          const balance = await mindCoinContract.balanceOf(userAccount);
          setMindCoinBalance(ethers.formatUnits(balance, 18)); // Assuming 18 decimals
          setFeedbackMessage("MindCoin balance updated.");
        } catch (error) {
          console.error("Error fetching MindCoin balance:", error);
          setFeedbackMessage("Error fetching MindCoin balance. See console.");
          setMindCoinBalance("Error");
        }
      }
    };

    fetchBalance();
  }, [mindCoinContract, userAccount, provider]); // Re-fetch if contract, account, or provider changes

  const connectWallet = async () => {
    if (!provider) {
      setFeedbackMessage("MetaMask is not available.");
      return;
    }
    try {
      setFeedbackMessage("Connecting to wallet...");
      // eth_requestAccounts returns an array of address strings
      const accounts = await provider.send("eth_requestAccounts", []); 
      if (accounts.length > 0) {
        // Pass the provider instance because state update might be async
        await handleAccountsChanged(accounts, provider); 
      } else {
        setFeedbackMessage("No accounts found. Please ensure your wallet is set up.");
      }
    } catch (error) {
      console.error("Error connecting wallet:", error);
      setFeedbackMessage(`Error connecting wallet: ${error.message || "Unknown error"}`);
    }
  };

  return (
    <div className="App">
      <header className="App-header">
        <h1>Welcome to OmniMind</h1>
        <p>The AI-Native, Decentralized Operating System Interface</p>
        {feedbackMessage && <p className="feedback"><em>{feedbackMessage}</em></p>}
      </header>
      <main>
        <section id="wallet-info">
          <h2>Wallet Information</h2>
          {userAccount ? (
            <>
              <p>Connected Account: {userAccount}</p>
              <p>MindCoin Balance: {mindCoinBalance} MIND</p>
            </>
          ) : (
            <button onClick={connectWallet} style={{ padding: '10px', marginTop: '5px' }}>
              Connect Wallet
            </button>
          )}
        </section>

        <section id="core-status">
          <h2>OmniMind Core Status:</h2>
          <p><em>(Connecting to local OmniMind Core services...)</em></p>
          <p id="core-message">Status: Not Yet Connected</p>
        </section>

        <section id="ai-interaction">
          <h2>AI Command Interface:</h2>
          <input type="text" placeholder="Type your command to OmniMind..." style={{ width: '80%', padding: '10px', margin: '10px 0' }} />
          <button onClick={() => alert('Command processing not yet implemented!')} style={{ padding: '10px' }}>
            Send Command
          </button>
        </section>

        {/* We'll keep the P2P info section as a placeholder for now */}
        <section id="p2p-info">
          <h2>P2P Network:</h2>
          <p>Status: <em>(Not connected to P2P network)</em></p>
          <p>Shared Resources: <em>0 Files / 0 CPU Cycles</em></p>
        </section>
      </main>
      <footer>
        <p>© {new Date().getFullYear()} OmniMind Project. You are the Trillionaire Founder.</p>
      </footer>
    </div>
  );
}

export default App;