# ZOS Dev Server - Auto-reload Daemon

Minimal development server that automatically rebuilds and restarts when source files change. Runs as a background daemon and replaces any existing instances.

## Usage

### Start Dev Server Daemon
```bash
./dev-auto-reload.sh
# or directly:
./target/debug/zos-dev-minimal
```

### Stop Dev Server Daemon
```bash
./stop-dev-server.sh
```

### Test Auto-reload
```bash
./test-auto-reload.sh
```

## Features

- **Background Daemon**: Automatically backgrounds itself on startup
- **Process Replacement**: Kills and replaces any existing dev server instances
- **File Watching**: Monitors `src/` directory recursively
- **Auto-rebuild**: Runs `cargo build` on file changes
- **Auto-restart**: Kills and restarts server process
- **Debouncing**: 500ms delay to avoid rapid rebuilds
- **PID Management**: Writes PID file for clean shutdown

## How It Works

1. **Process Cleanup**: Kills any existing dev server processes
2. **Daemon Fork**: Starts itself with `--daemon` flag in background
3. **PID File**: Writes process ID to `/tmp/zos-dev.pid`
4. **File Watcher**: Uses `notify` crate to watch `src/` directory
5. **Change Detection**: Triggers on any file modification
6. **Rebuild Process**:
   - Kills existing server process
   - Runs `cargo build --bin zos_server --quiet`
   - Starts new server process if build succeeds
7. **Debouncing**: Ignores rapid successive changes

## Output

```
🔥 Starting ZOS Dev Server daemon with auto-reload...
✅ Dev server started in background (PID: 12345)
👀 Watching src/ for changes...
🌐 Server: http://localhost:8080
🛑 Stop with: pkill -f zos-dev-minimal
```

## Process Management

- **Start**: `./target/debug/zos-dev-minimal`
- **Stop**: `./stop-dev-server.sh` or `pkill -f zos-dev-minimal`
- **Status**: `curl http://localhost:8080/health`
- **PID File**: `/tmp/zos-dev.pid`

## Daemon Architecture

```rust
fn main() {
    if args[1] == "--daemon" {
        daemonize();  // Run file watcher loop
    } else {
        kill_existing_processes();  // Clean up
        spawn_daemon();             // Fork with --daemon
    }
}
```

## Error Handling

- **Build Failures**: Silent (daemon continues running)
- **Start Failures**: Silent (daemon continues running)
- **Process Management**: Properly kills old processes before starting new ones
- **PID File**: Cleaned up on normal shutdown

## Comparison with Foreground Version

| Feature | Daemon | Foreground |
|---------|--------|------------|
| Background execution | ✅ | ❌ |
| Process replacement | ✅ | ❌ |
| PID management | ✅ | ❌ |
| Clean shutdown | ✅ | ❌ |
| Terminal output | Minimal | Verbose |
| Lines of code | ~90 | ~60 |

The daemon version adds robust process management while maintaining the core auto-reload functionality.
