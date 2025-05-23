/* global BigInt */
import React, { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import './App.css';
import { mindCoinAddress, mindCoinABI } from './contractInfo';

function App() {
  const [userAccount, setUserAccount] = useState(null);
  const [mindCoinBalance, setMindCoinBalance] = useState("0");
  const [provider, setProvider] = useState(null);
  const [signer, setSigner] = useState(null);
  const [mindCoinContract, setMindCoinContract] = useState(null);
  const [feedbackMessage, setFeedbackMessage] = useState("");
  const [transferToAddress, setTransferToAddress] = useState("");
  const [transferAmount, setTransferAmount] = useState("");
  const [coreCommand, setCoreCommand] = useState("");
  const [coreResponse, setCoreResponse] = useState(null);
  const [isCoreLoading, setIsCoreLoading] = useState(false);
  const [ipfsFileToAdd, setIpfsFileToAdd] = useState(null);
  const [cidToCat, setCidToCat] = useState("");
  const [ipfsAddPath, setIpfsAddPath] = useState("");
  const [ipfsCatCid, setIpfsCatCid] = useState("");

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


  const handleTransferMindCoin = async () => {
    if (!mindCoinContract || !userAccount || !signer) {
      setFeedbackMessage("Please connect your wallet first.");
      return;
    }
    if (!ethers.isAddress(transferToAddress)) {
      setFeedbackMessage("Invalid recipient address.");
      return;
    }
    if (isNaN(parseFloat(transferAmount)) || parseFloat(transferAmount) <= 0) {
      setFeedbackMessage("Invalid transfer amount.");
      return;
    }

    try {
      setFeedbackMessage(`Initiating transfer of ${transferAmount} MIND to ${transferToAddress.substring(0,6)}...`);
      
      // Ensure we are on Sepolia
      const network = await provider.getNetwork();
      if (network.chainId !== BigInt(11155111)) {
        setFeedbackMessage("Please switch to the Sepolia test network to transfer MindCoin.");
        return;
      }

      const amountInWei = ethers.parseUnits(transferAmount, 18); // Convert to smallest unit (18 decimals)
      
      // Estimate gas (optional, but good for UX to catch potential out-of-gas errors early)
      // try {
      //   const estimatedGas = await mindCoinContract.transfer.estimateGas(transferToAddress, amountInWei);
      //   console.log("Estimated gas for transfer:", estimatedGas.toString());
      // } catch (gasError) {
      //   console.error("Gas estimation failed:", gasError);
      //   setFeedbackMessage(`Transaction likely to fail: ${gasError.reason || gasError.message}`);
      //   return;
      // }

      const tx = await mindCoinContract.transfer(transferToAddress, amountInWei);
      setFeedbackMessage(`Transfer transaction sent: ${tx.hash.substring(0,10)}... Waiting for confirmation...`);

      await tx.wait(); // Wait for the transaction to be mined

      setFeedbackMessage(`Successfully transferred ${transferAmount} MIND to ${transferToAddress.substring(0,6)}...`);
      
      // Clear input fields
      setTransferToAddress("");
      setTransferAmount("");

      // Refresh balance (could also optimistically update)
      const balance = await mindCoinContract.balanceOf(userAccount);
      setMindCoinBalance(ethers.formatUnits(balance, 18));

    } catch (error) {
      console.error("Error transferring MindCoin:", error);
      let userMessage = "Error transferring MindCoin.";
      if (error.reason) {
        userMessage += ` Reason: ${error.reason}`;
      } else if (error.message) {
          userMessage += ` Details: ${error.message.substring(0,100)}...`;
      }
      setFeedbackMessage(userMessage);
    }
  };

  const sendCommandToCore = async () => {
    if (!coreCommand.trim()) {
      setFeedbackMessage("Please enter a command for OmniMind Core.");
      return;
    }

    setIsCoreLoading(true);
    setCoreResponse(null); // Clear previous response
    setFeedbackMessage("Sending command to OmniMind Core...");

    try {
      const response = await fetch("http://localhost:3030/command", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ raw_command: coreCommand }),
      });

      if (!response.ok) {
        // Try to get error message from server if possible, or use status text
        let errorMsg = `HTTP error! status: ${response.status}`;
        try {
          const errorData = await response.json();
          errorMsg = errorData.message || errorMsg; // Assuming server sends JSON error with a message field
        } catch (e) {
          // Ignore if error response is not JSON
        }
        throw new Error(errorMsg);
      }

      const data = await response.json(); // This is our CommandResponse struct
      setCoreResponse(data);
      
      if (data.status === "success") {
          setFeedbackMessage("OmniMind Core processed command successfully.");
      } else {
          setFeedbackMessage(`OmniMind Core returned an error: ${data.message}`);
      }
      setCoreCommand(""); // Clear input field after sending

    } catch (error) {
      console.error("Error sending command to Core API:", error);
      setFeedbackMessage(`Error communicating with OmniMind Core: ${error.message}`);
      setCoreResponse({ status: "error", message: `Network or server error: ${error.message}`, data: null });
    } finally {
      setIsCoreLoading(false);
    }
  };

  const handleIpfsAdd = async () => {
    if (!ipfsAddPath.trim()) {
      setFeedbackMessage("Please enter a local file path to add to IPFS.");
      return;
    }
    // Construct the raw command string for the backend
    const rawCommand = `ipfs_add ${ipfsAddPath}`;
    
    setIsCoreLoading(true);
    setCoreResponse(null);
    setFeedbackMessage("Adding file to IPFS via OmniMind Core...");

    try {
      const response = await fetch("http://localhost:3030/command", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ raw_command: rawCommand }),
      });
      // ... (identical response handling logic as in sendCommandToCore) ...
      if (!response.ok) {
        let errorMsg = `HTTP error! status: ${response.status}`;
        try { const errorData = await response.json(); errorMsg = errorData.message || errorMsg; } catch (e) { /* ignore */ }
        throw new Error(errorMsg);
      }
      const data = await response.json();
      setCoreResponse(data);
      if (data.status === "success") {
          setFeedbackMessage("IPFS add command processed successfully.");
          setIpfsAddPath(""); // Clear input
      } else {
          setFeedbackMessage(`IPFS add command error: ${data.message}`);
      }
    } catch (error) {
      console.error("Error in handleIpfsAdd:", error);
      setFeedbackMessage(`Error adding to IPFS: ${error.message}`);
      setCoreResponse({ status: "error", message: `Network or server error: ${error.message}`, data: null });
    } finally {
      setIsCoreLoading(false);
    }
  };

  const handleIpfsCat = async () => {
    if (!ipfsCatCid.trim()) {
      setFeedbackMessage("Please enter an IPFS CID to retrieve.");
      return;
    }
    const rawCommand = `ipfs_cat ${ipfsCatCid}`;

    setIsCoreLoading(true);
    setCoreResponse(null);
    setFeedbackMessage("Getting file from IPFS via OmniMind Core...");

    try {
      const response = await fetch("http://localhost:3030/command", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ raw_command: rawCommand }),
      });
      // ... (identical response handling logic as in sendCommandToCore) ...
      if (!response.ok) {
        let errorMsg = `HTTP error! status: ${response.status}`;
        try { const errorData = await response.json(); errorMsg = errorData.message || errorMsg; } catch (e) { /* ignore */ }
        throw new Error(errorMsg);
      }
      const data = await response.json();
      setCoreResponse(data);
      if (data.status === "success") {
          setFeedbackMessage("IPFS cat command processed successfully.");
          // setIpfsCatCid(""); // Optionally clear CID input, or leave it for re-query
      } else {
          setFeedbackMessage(`IPFS cat command error: ${data.message}`);
      }
    } catch (error) {
      console.error("Error in handleIpfsCat:", error);
      setFeedbackMessage(`Error getting from IPFS: ${error.message}`);
      setCoreResponse({ status: "error", message: `Network or server error: ${error.message}`, data: null });
    } finally {
      setIsCoreLoading(false);
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
        {userAccount && (
          <section id="transfer-mindcoin">
            <h2>Transfer MindCoin</h2>
            <div>
              <input
                type="text"
                placeholder="Recipient Address (0x...)"
                value={transferToAddress}
                onChange={(e) => setTransferToAddress(e.target.value)}
                style={{ width: 'calc(80% - 20px)', padding: '10px', margin: '5px 0' }}
              />
            </div>
            <div>
              <input
                type="text" // Use text for now, can add number validation later
                placeholder="Amount (e.g., 100)"
                value={transferAmount}
                onChange={(e) => setTransferAmount(e.target.value)}
                style={{ width: 'calc(80% - 20px)', padding: '10px', margin: '5px 0' }}
              />
            </div>
            <button onClick={handleTransferMindCoin} style={{ padding: '10px', marginTop: '10px' }}>
              Transfer MindCoin
            </button>
          </section>
        )} 
        <section id="core-status">
          <h2>OmniMind Core Status:</h2>
          <p><em>(Connecting to local OmniMind Core services...)</em></p>
          <p id="core-message">Status: Not Yet Connected</p>
        </section>

        <section id="ai-interaction">
          <h2>OmniMind Core Interface</h2>
          <div style={{ marginBottom: '10px' }}>
            <input
              type="text"
              placeholder="Type your command to OmniMind Core (e.g., help, ls, create_note title)"
              value={coreCommand}
              onChange={(e) => setCoreCommand(e.target.value)}
              onKeyPress={(e) => { if (e.key === 'Enter') sendCommandToCore(); }}
              style={{ width: 'calc(80% - 20px)', padding: '10px', marginRight: '5px' }}
              disabled={isCoreLoading}
            />
            <button onClick={sendCommandToCore} style={{ padding: '10px' }} disabled={isCoreLoading}>
              {isCoreLoading ? "Sending..." : "Send to Core"}
            </button>
          </div>
        </section>

        {/* NEW SECTION FOR IPFS OPERATIONS */}
        <section id="ipfs-operations">
          <h2>IPFS Operations (via OmniMind Core)</h2>
          
          {/* IPFS Add */}
          <div style={{ marginTop: '15px', padding: '10px', border: '1px dashed #007bff' }}>
            <h4>Add File to IPFS</h4>
            <p><em>Type the full local path to a file that the OmniMind Core server can access.</em></p>
            <input
              type="text"
              placeholder="e.g., ./my_local_file.txt or C:\path\to\file.txt"
              value={ipfsAddPath}
              onChange={(e) => setIpfsAddPath(e.target.value)}
              onKeyPress={(e) => { if (e.key === 'Enter') handleIpfsAdd(); }}
              style={{ width: 'calc(70% - 10px)', padding: '10px', marginRight: '5px' }}
              disabled={isCoreLoading}
            />
            <button 
              onClick={handleIpfsAdd}
              style={{ padding: '10px' }} 
              disabled={isCoreLoading || !ipfsAddPath.trim()}
            >
              {isCoreLoading && ipfsAddPath /* crude check if this button caused loading */ ? "Adding..." : "Add to IPFS"}
            </button>
            <p style={{fontSize: '0.8em', color: 'gray'}}>Ensure `omnimind-core` has permissions to read the file.</p>
          </div>

          {/* IPFS Cat */}
          <div style={{ marginTop: '15px', padding: '10px', border: '1px dashed #28a745' }}>
            <h4>Get File from IPFS (Cat)</h4>
            <input
              type="text"
              placeholder="Enter IPFS CID (e.g., QmYourCID...)"
              value={ipfsCatCid}
              onChange={(e) => setIpfsCatCid(e.target.value)}
              onKeyPress={(e) => { if (e.key === 'Enter') handleIpfsCat(); }}
              style={{ width: 'calc(70% - 10px)', padding: '10px', marginRight: '5px' }}
              disabled={isCoreLoading}
            />
            <button 
              onClick={handleIpfsCat}
              style={{ padding: '10px' }} 
              disabled={isCoreLoading || !ipfsCatCid.trim()}
            >
              {isCoreLoading && ipfsCatCid /* crude check */ ? "Getting..." : "Get from IPFS"}
            </button>
          </div>
        </section> {/* End of ipfs-operations section */}

        {/* Display area for core responses (this will now show results from general commands AND IPFS ops) */}
        {coreResponse && (
          <div id="core-response-area" style={{ marginTop: '15px', padding: '10px', border: '1px solid #ccc', whiteSpace: 'pre-wrap', textAlign: 'left', maxHeight: '300px', overflowY: 'auto' }}>
            <h4>Core Response:</h4>
            <p><strong>Status:</strong> {coreResponse.status}</p>
            <p><strong>Message:</strong></p>
            <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>{coreResponse.message}</pre>
            {coreResponse.data && (
              <div>
                {/* NEW: Display NLU Confidence if available from nlu_details, nlu_confidence, or nlu_result */}
                {coreResponse.data && 
                 ((coreResponse.data.nlu_details && coreResponse.data.nlu_details.nlu_confidence) || 
                  coreResponse.data.nlu_confidence || 
                  (coreResponse.data.nlu_result && coreResponse.data.nlu_result.confidence)) && (
                  <p style={{ fontStyle: 'italic', color: '#555', fontSize: '0.9em' }}>
                    (Interpreted via NLU with confidence: 
                    {
                      ((
                        (coreResponse.data.nlu_details && coreResponse.data.nlu_details.nlu_confidence) || 
                        coreResponse.data.nlu_confidence || 
                        (coreResponse.data.nlu_result && coreResponse.data.nlu_result.confidence) || 
                        0 // Fallback to 0 if none are found
                      ) * 100).toFixed(1)
                    }%)
                  </p>
                )}

                <p><strong>Data:</strong></p>
                <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                  {JSON.stringify(coreResponse.data, null, 2)}
                </pre>
              </div>
            )}
          </div>
        )}
        
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