# ZOS Frontend - Wallet Connect Interface
# Deploy to Vercel at zos-frontend.vercel.app

## Static Site Structure
```
public/
├── index.html          # Main landing page
├── wallet-connect.js   # WalletConnect integration
├── zos-api.js         # ZOS network API client
└── styles.css         # Styling

api/
├── wallet-balance.js  # Serverless function to check balances
└── network-status.js  # Network status endpoint
```

## Cloudflare Worker (Optional Proxy)
```javascript
// cloudflare-worker.js
export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // Serve static frontend
    if (url.pathname === '/' || url.pathname.startsWith('/static/')) {
      return fetch('https://zos-frontend.vercel.app' + url.pathname);
    }

    // Proxy API calls to Oracle instance
    if (url.pathname.startsWith('/api/')) {
      const oracleUrl = `http://${env.ORACLE_IP}${url.pathname}`;
      return fetch(oracleUrl, {
        method: request.method,
        headers: request.headers,
        body: request.body
      });
    }

    return new Response('Not found', { status: 404 });
  }
}
```

## Wallet Connect Integration
```html
<!DOCTYPE html>
<html>
<head>
    <title>ZOS Network - solfunmeme</title>
    <script src="https://unpkg.com/@walletconnect/web3-provider@1.8.0/dist/umd/index.min.js"></script>
</head>
<body>
    <div id="app">
        <h1>🌐 ZOS Network Access</h1>
        <div id="wallet-status">
            <button id="connect-wallet">Connect Wallet</button>
        </div>
        <div id="network-info" style="display:none;">
            <h2>Your Access Level</h2>
            <div id="tier-info"></div>
            <div id="balance-info"></div>
            <div id="rate-limit-info"></div>
        </div>
    </div>

    <script>
        let walletAddress = null;
        let accessTier = 'public';

        async function connectWallet() {
            try {
                // WalletConnect integration
                const provider = new WalletConnectProvider.default({
                    rpc: {
                        101: "https://api.mainnet-beta.solana.com"
                    }
                });

                await provider.enable();
                walletAddress = provider.accounts[0];

                // Check balance and tier
                const response = await fetch('/api/wallet-info', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ address: walletAddress })
                });

                const walletInfo = await response.json();
                updateUI(walletInfo);

            } catch (error) {
                console.error('Wallet connection failed:', error);
            }
        }

        function updateUI(walletInfo) {
            document.getElementById('wallet-status').innerHTML = `
                <p>✅ Connected: ${walletAddress.substring(0, 8)}...</p>
            `;

            document.getElementById('tier-info').innerHTML = `
                <p><strong>Access Tier:</strong> ${walletInfo.tier.toUpperCase()}</p>
            `;

            document.getElementById('balance-info').innerHTML = `
                <p><strong>Balance:</strong> ${walletInfo.balance.toFixed(2)} SOL</p>
            `;

            document.getElementById('rate-limit-info').innerHTML = `
                <p><strong>Rate Limit:</strong> ${walletInfo.rateLimit}</p>
            `;

            document.getElementById('network-info').style.display = 'block';
        }

        document.getElementById('connect-wallet').onclick = connectWallet;
    </script>
</body>
</html>
```

## Deployment Commands
```bash
# Deploy to Vercel
npx vercel --prod

# Configure Cloudflare Worker (optional)
wrangler publish

# Update Oracle instance with frontend URL
terraform apply -var="frontend_url=https://zos-frontend.vercel.app"
```

## Access Tiers
- **Root**: Oracle PEM key holders - Unlimited access
- **Whales**: 1M+ SOL balance - 100 requests/minute
- **Holders**: 1K+ SOL balance - 50 requests/minute
- **Public**: No wallet - 10 requests/minute

## Rate Limiting Implementation
- **Frontend**: Wallet Connect → Balance check → Tier assignment
- **Backend**: nginx auth_request → Python validator → iptables marking → tc QoS
- **Network**: Traffic shaping based on wallet tier
