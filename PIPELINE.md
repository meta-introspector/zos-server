# ZOS CI/CD Pipeline Documentation

## Overview
The ZOS server implements a complete CI/CD pipeline with automated deployment from development through QA to production, with git-based updates and service management.

## Architecture

### Service Topology
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Dev Server    │───▶│   QA Service    │───▶│ Production Svc  │
│   Port 8080     │    │   Port 8082     │    │   Port 8084     │
│   Local Build   │    │   Git: qa       │    │   Git: stable   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Directory Structure
```
/opt/zos-test-qa/          # QA environment
├── .git/                  # qa branch checkout
├── zos-minimal-server/    # QA server code
└── data/                  # QA runtime data

/opt/zos-test-production/  # Production environment
├── .git/                  # stable branch checkout
├── zos-minimal-server/    # Production server code
└── data/                  # Production runtime data
```

## Pipeline Endpoints

### Development Server (localhost:8080)

#### Service Management
- `POST /install/qa-service` - Install QA service with dedicated user
- `POST /manage/qa/update` - Trigger QA server git update and restart

#### CI/CD Pipeline
- `POST /deploy/dev-to-staging` - Deploy current dev code to staging
- `POST /deploy/staging-to-prod` - Promote staging build to production
- `POST /deploy/rollout` - Update stable branch for client distribution

#### Git Integration
- `POST /webhook/git` - Handle git webhook notifications
- `POST /poll-git` - Poll for git updates on specified branch

### QA Server (localhost:8082)

#### Self-Management
- `POST /update-self` - Self-update from git and restart service
- `GET /health` - Health check with git commit information
- `GET /api/status` - Detailed service status

### Production Server (localhost:8084)

#### Standard Operations
- `GET /health` - Production health check
- `GET /api/status` - Production service status
- All standard ZOS server endpoints

## Git Branch Strategy

### Branch Roles
- **main** - Active development branch
- **qa** - QA testing and validation branch
- **stable** - Production-ready releases

### Update Flow
```
Developer commits → main branch
       ↓
QA pulls from → qa branch
       ↓
Production pulls from → stable branch
       ↓
Clients pull from → stable branch
```

## Service Management

### System Users
- `zos-qa` - Dedicated user for QA service
- `zos-prod` - Dedicated user for production service

### Systemd Services
- `zos-qa.service` - QA server service
- `zos-production.service` - Production server service

### Service Configuration
```ini
[Unit]
Description=ZOS QA Server - Resource Tracing
After=network.target

[Service]
Type=simple
User=zos-qa
Group=zos-qa
WorkingDirectory=/opt/zos-test-qa
ExecStart=/usr/local/bin/zos-qa-server
Restart=always
RestartSec=5

Environment=ZOS_HTTP_PORT=8082
Environment=ZOS_DATA_DIR=/opt/zos-test-qa/data
```

## Deployment Scripts

### Key Scripts
- `deploy-pipeline.sh` - Full pipeline deployment
- `update-prod.sh` - Production update script (called by QA)
- `test-pipeline.sh` - Pipeline testing and validation

### Update Process
1. **Development**: Local changes and testing
2. **Commit**: Push changes to main branch
3. **QA Trigger**: Call `/manage/qa/update` endpoint
4. **QA Update**: QA service pulls qa branch, rebuilds, restarts
5. **QA Validation**: Automated testing of QA environment
6. **Production Trigger**: QA calls production update script
7. **Production Update**: Production pulls stable branch, rebuilds
8. **Client Rollout**: Stable branch updated for client distribution

## Security Model

### Isolation
- Separate users for each environment
- Dedicated directories with proper ownership
- Service-level isolation via systemd

### Update Authorization
- QA service can trigger production updates via sudo script
- Production updates require specific script execution
- Git-based updates ensure code integrity

### Network Security
- Services bound to localhost by default
- Port-based service isolation
- Health check endpoints for monitoring

## Testing and Validation

### Automated Testing
- Health checks at each stage
- API endpoint validation
- Service restart verification
- Git status confirmation

### Manual Testing
- `test-pipeline.sh` - Comprehensive pipeline test
- Individual endpoint testing
- Service status monitoring

## Configuration

### Environment Variables
```bash
ZOS_HTTP_PORT=8080        # Dev server port
ZOS_QA_PORT=8082          # QA server port
ZOS_PROD_PORT=8084        # Production server port
ZOS_DATA_DIR=/opt/zos/data # Data directory
```

### Git Configuration
- Remote: https://github.com/meta-introspector/zos-server.git
- Branches: main, qa, stable
- Update strategy: Pull and rebuild on change

## Monitoring and Logging

### Health Endpoints
- All services provide `/health` endpoint
- Git commit information included in health status
- Service uptime and status reporting

### Logging
- Systemd journal integration
- Service-specific log files
- Build and deployment logging

## Troubleshooting

### Common Issues
1. **Service Won't Start**: Check systemd status and logs
2. **Git Update Fails**: Verify branch exists and permissions
3. **Build Failures**: Check Rust toolchain and dependencies
4. **Port Conflicts**: Verify no other services on required ports

### Debug Commands
```bash
# Check service status
sudo systemctl status zos-qa.service

# View service logs
sudo journalctl -u zos-qa.service -f

# Test endpoints
curl http://localhost:8082/health

# Manual git update
sudo -u zos-qa bash -c "cd /opt/zos-test-qa && git pull origin qa"
```

## Hash Verification and Deployment

### Hash-Based Deployment
ZOS now implements cryptographic verification of deployments using git commit hashes and binary hashes to ensure integrity across the pipeline.

#### Verification Endpoints
- `GET /health` - Returns git hash, binary hash, and verification status
- `POST /deploy/verify-hash/:hash` - Deploy specific git commit hash with verification

#### Hash Flow
```
1. Dev commits → Git Hash: abc123...
2. QA fetches abc123, builds, verifies binary hash
3. Prod deploys only verified hash combinations
4. All services report current hashes in /health
```

#### Current Verified Hash
- **Git**: `81038be429724d9e980698e4d3dd7eddeedd8802`
- **Short**: `81038be`
- **Message**: "Add git hash and binary hash verification"

### Usage
```bash
# Get current deployment hash
HASH=$(curl -s http://localhost:8080/health | jq -r .git.commit)

# Deploy to QA with verification
curl -X POST "http://localhost:8080/deploy/verify-hash/$HASH"

# Verify QA deployment
curl -s http://localhost:8082/health | jq '.git.commit_short'
```
