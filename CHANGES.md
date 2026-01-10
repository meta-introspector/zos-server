# ZOS Server Changes Log

## Current Changes (Uncommitted)

### Modified Files:
- `update-prod.sh`: Changed production directory from `/opt/zos-production` to `/opt/zos-test-production`
- `zos-minimal-server/src/main.rs`: Updated QA installation paths from `/opt/zos-qa` to `/opt/zos-test-qa`

### Changes Summary:
1. **Production Path Update**: Modified production deployment to use test directory structure
2. **QA Path Update**: Updated QA service configuration to use test directory structure
3. **Service Configuration**: Updated systemd service working directory and data directory paths

### Impact:
- Isolates test deployments from production systems
- Maintains separate directory structure for testing pipeline
- Ensures QA and production environments don't conflict

### Status:
- 2 commits ahead of origin/main
- ZOS processes stopped (PIDs: 1317859, 1318091)
- ZOS QA service stopped and inactive

## Hash Verification System

### Current Deployment Hashes
- **Git Hash**: `81038be429724d9e980698e4d3dd7eddeedd8802` (short: `81038be`)
- **Commit**: "Add git hash and binary hash verification with proper borrow handling"
- **Date**: 2026-01-10T16:09:47+00:00

### Hash Verification Features
- **Health Endpoint**: `/health` now includes git commit hash and binary hash
- **Hash Deployment**: `/deploy/verify-hash/:hash` deploys specific git hash
- **Binary Verification**: SHA256 hash of running binary included in health check
- **Git Integrity**: Verifies git hash exists before deployment

### Pipeline Hash Flow
```
Dev (git: 81038be) → QA (verify: 81038be + build) → Prod (verify: binary hash matches)
```

### Verification Commands
```bash
# Check current server hashes
curl -s http://localhost:8080/health | jq '.git + .binary'

# Deploy specific git hash to QA
curl -X POST http://localhost:8080/deploy/verify-hash/81038be429724d9e980698e4d3dd7eddeedd8802

# Verify QA has correct hash
curl -s http://localhost:8082/health | jq '.git.commit_short'
```

### Available Endpoints:

#### Development Server (Port 8080)
- `/install/qa-service` - Install QA service from git branch
- `/manage/qa/update` - Trigger QA server update from git
- `/deploy/dev-to-staging` - Deploy dev changes to staging
- `/deploy/staging-to-prod` - Promote staging to production
- `/deploy/rollout` - Update stable branch for client rollout

#### QA Server (Port 8082)
- `/health` - Health check with git status
- `/update-self` - Self-update from git branch
- `/api/status` - Service status information

#### Production Server (Port 8084)
- `/health` - Production health check
- Standard ZOS server endpoints

### Git Branch Strategy:
- `main` - Development branch
- `qa` - QA testing branch
- `stable` - Production/client rollout branch

### Service Management:
- QA service runs as `zos-qa` user in `/opt/zos-test-qa`
- Production service runs as `zos-prod` user in `/opt/zos-test-production`
- Services managed via systemd (`zos-qa.service`, `zos-production.service`)

### Update Process:
1. Dev commits changes locally
2. Dev triggers QA update via `/manage/qa/update`
3. QA service pulls from `qa` branch and rebuilds
4. QA validates changes
5. Production update triggered via `update-prod.sh`
6. Production pulls from `stable` branch
