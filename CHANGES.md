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

## Current CI/CD Pipeline Architecture

### Pipeline Flow:
```
Dev (Local) → QA Service → Production Service
    ↓             ↓              ↓
Port 8080    Port 8082     Port 8084
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
