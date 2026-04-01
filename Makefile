.PHONY: build build-minimal test deploy restart status setup

# Build everything via nix
build:
	nix build .#zos-server

build-minimal:
	nix build .#zos-minimal-server

test:
	nix develop -c cargo test --all-features

# Deploy minimal server to systemd
deploy: build-minimal
	systemctl --user stop zos-minimal-server || true
	cp -f result/bin/zos-minimal-server ~/.local/bin/zos-minimal-server
	systemctl --user daemon-reload
	systemctl --user start zos-minimal-server
	@echo "Deployed. Check: make status"

restart:
	systemctl --user restart zos-minimal-server

status:
	systemctl --user status zos-minimal-server
	@echo "---"
	@curl -sf http://127.0.0.1:8081/health | head -1 || echo "health check failed"

setup:
	@echo "Open http://127.0.0.1:8081/setup in your browser"
	@curl -sf http://127.0.0.1:8081/setup > /dev/null && echo "Setup wizard is live" || echo "Server not running — try: make restart"
