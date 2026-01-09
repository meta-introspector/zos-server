#!/bin/bash

# ZOS Free Tier Demo Script
# Shows the block-based port allocation and service marketplace

echo "🎮 ZOS Free Tier Demo - Block-based Port Allocation"
echo "=================================================="

# Simulate user getting a port for one block
echo ""
echo "👤 User alice_123 requests a port..."
echo "🔌 Port 20001 allocated for block 12345 (expires block 12346)"
echo "⏰ Block duration: 400ms (Solana speed)"

echo ""
echo "🎯 Available Free Tier Services:"
echo "1. 🥧 pi_calculator - Calculate π using Leibniz formula (1 credit)"
echo "2. 🐰 fibonacci_meme - Fibonacci with rabbit meme (2 credits)"
echo "3. 🎭 prime_poetry - Prime numbers with poetic flair (1 credit)"

echo ""
echo "🚀 Executing pi_calculator..."
echo "📝 Code: fn calculate_pi(iterations: u32) -> f64 { ... }"
echo "📊 Result: π ≈ 3.1415926536"
echo "💰 Cost: 1 credit"

echo ""
echo "🤝 alice_123 shares port 20001 with bob_456..."
echo "✅ Port shared successfully"

echo ""
echo "🏪 alice_123 lists port for resale:"
echo "   Port: 20001"
echo "   Price: 5 credits"
echo "   Duration: 1 block"
echo "   Description: 'Pi calculation service with sharing'"

echo ""
echo "💰 charlie_789 buys access to port 20001..."
echo "✅ Purchase successful - now 3 users sharing port 20001"

echo ""
echo "📊 Current Server Status:"
echo "   Block: 12345"
echo "   Active Ports: 15/50 (30% capacity)"
echo "   Marketplace Listings: 8"
echo "   Total Users This Block: 42"

echo ""
echo "⏰ Block 12346 begins..."
echo "🔄 Port 20001 expires - all users need new allocation"
echo "🆕 New ports available for next block"

echo ""
echo "🎉 Free Tier Value Proposition:"
echo "   ✨ Get temporary compute access"
echo "   🤝 Share with friends"
echo "   💰 Resell unused capacity"
echo "   🎮 Fun services (pi, fibonacci, primes)"
echo "   ⚡ Real-time block-based allocation"
echo "   🏪 Built-in marketplace"

echo ""
echo "🚀 Try it yourself:"
echo "   curl -X POST localhost:3000/allocate-port"
echo "   curl -X POST localhost:3000/execute/pi_calculator"
echo "   curl -X POST localhost:3000/share-port -d '{\"share_with\":\"friend_id\"}'"
echo "   curl -X POST localhost:3000/list-for-sale -d '{\"price\":5}'"
