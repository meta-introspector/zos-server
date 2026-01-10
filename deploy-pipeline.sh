#!/bin/bash
# ZOS CI/CD Pipeline - Deploy from local dev to production

set -e

echo "🚀 ZOS CI/CD Pipeline"
echo "📂 Local dev → Staging → Production → Client rollout"
echo ""

# Step 1: Push local changes to git
echo "1️⃣ Pushing local changes to git..."
git add .
git commit -m "Auto-deploy: $(date)" || echo "No changes to commit"
git push origin main

# Step 2: Deploy to staging (dev server port 8080)
echo "2️⃣ Deploying to staging server..."
curl -X POST http://localhost:8080/deploy/dev-to-staging

# Wait for staging deployment
sleep 10

# Step 3: Test staging server
echo "3️⃣ Testing staging server..."
STAGING_HEALTH=$(curl -s http://localhost:8080/health | jq -r .status)
if [ "$STAGING_HEALTH" != "healthy" ]; then
    echo "❌ Staging health check failed"
    exit 1
fi

# Test installer endpoint
curl -s http://localhost:8080/install.sh | head -5 | grep -q "ZOS Universal Installer"
if [ $? -eq 0 ]; then
    echo "✅ Staging tests passed"
else
    echo "❌ Staging installer test failed"
    exit 1
fi

# Step 4: Deploy to production (prod server port 8081)
echo "4️⃣ Deploying to production server..."
curl -X POST http://localhost:8080/deploy/staging-to-prod

# Wait for production deployment
sleep 10

# Step 5: Test production server
echo "5️⃣ Testing production server..."
PROD_HEALTH=$(curl -s http://localhost:8081/health | jq -r .status)
if [ "$PROD_HEALTH" != "healthy" ]; then
    echo "❌ Production health check failed"
    exit 1
fi

echo "✅ Production tests passed"

# Step 6: Rollout to clients (update stable branch)
echo "6️⃣ Rolling out to clients..."
curl -X POST http://localhost:8080/deploy/rollout

echo ""
echo "🎉 CI/CD Pipeline Complete!"
echo "   📊 Staging:    http://localhost:8080"
echo "   🏭 Production: http://localhost:8081"
echo "   🌐 Clients:    Will pull from stable branch"
