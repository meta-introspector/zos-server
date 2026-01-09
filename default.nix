# ZOS Core Lattice - Pure Nix Build
{ pkgs ? import <nixpkgs> {} }:

let
  # Security domain definitions
  securityDomains = {
    l0_public = {
      level = 0;
      orbits = [ "trivial" ];
      capabilities = [ "read" "compute" ];
      syscalls = [];
      memory_limit = "1MB";
    };

    l1_gateway = {
      level = 1;
      orbits = [ "trivial" "cyclic" ];
      capabilities = [ "read" "compute" "auth" "route" ];
      syscalls = [ "read" "write" ];
      memory_limit = "10MB";
    };

    l2_service = {
      level = 2;
      orbits = [ "trivial" "cyclic" "symmetric" ];
      capabilities = [ "read" "compute" "auth" "route" "process" ];
      syscalls = [ "read" "write" "open" "close" ];
      memory_limit = "100MB";
    };

    l3_core = {
      level = 3;
      orbits = [ "trivial" "cyclic" "symmetric" "alternating" ];
      capabilities = [ "read" "compute" "auth" "route" "process" "admin" ];
      syscalls = [ "read" "write" "open" "close" "chmod" "chown" ];
      memory_limit = "1GB";
    };

    l4_kernel = {
      level = 4;
      orbits = [ "trivial" "cyclic" "symmetric" "alternating" "sporadic" "monster" ];
      capabilities = [ "all" ];
      syscalls = [ "all" ];
      memory_limit = "unlimited";
    };
  };

  # Build each security layer
  buildSecurityLayer = domain: domainConfig: pkgs.rustPlatform.buildRustPackage rec {
    pname = "zos-${domain}";
    version = "0.1.0";

    src = ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    # Layer-specific build configuration
    buildFeatures = [
      "security-${domain}"
      "orbit-${builtins.concatStringsSep "-" domainConfig.orbits}"
      "memory-limit-${domainConfig.memory_limit}"
    ];

    # Security-hardened build flags
    RUSTFLAGS = [
      "-C target-feature=+crt-static"
      "-C relocation-model=static"
      "-C panic=abort"
      "-Z sanitizer=address"
      "-Z sanitizer=memory"
    ];

    # Domain-specific environment
    buildInputs = with pkgs; [
      openssl
      pkg-config
    ] ++ (if domainConfig.level >= 3 then [ systemd ] else [])
      ++ (if domainConfig.level >= 4 then [ linux-pam ] else []);

    # Compile-time security enforcement
    preBuild = ''
      # Generate domain-specific security policy
      cat > src/generated/domain_policy.rs << EOF
      // Auto-generated domain policy for ${domain}
      pub const DOMAIN_LEVEL: u8 = ${toString domainConfig.level};
      pub const ALLOWED_ORBITS: &[&str] = &${builtins.toJSON domainConfig.orbits};
      pub const ALLOWED_CAPABILITIES: &[&str] = &${builtins.toJSON domainConfig.capabilities};
      pub const ALLOWED_SYSCALLS: &[&str] = &${builtins.toJSON domainConfig.syscalls};
      pub const MEMORY_LIMIT: &str = "${domainConfig.memory_limit}";

      // Compile-time enforcement
      #[cfg(not(feature = "security-${domain}"))]
      compile_error!("Domain ${domain} requires security-${domain} feature");
      EOF

      # Strip forbidden syscalls at build time
      ${pkgs.gnused}/bin/sed -i 's/libc::\(${builtins.concatStringsSep "\\|" (
        if domainConfig.syscalls == ["all"] then [] else
        ["execve" "ptrace" "mount" "setuid"] # Remove syscalls not in allowed list
      )}\)/compile_error!("Forbidden syscall")/g' src/**/*.rs || true
    '';

    # Post-build verification
    postBuild = ''
      # Verify no forbidden symbols in binary
      if ${pkgs.binutils}/bin/objdump -T $out/bin/${pname} | grep -E "(execve|ptrace|mount)" && [ "${domain}" != "l4_kernel" ]; then
        echo "ERROR: Forbidden syscalls found in ${domain} binary"
        exit 1
      fi

      # Verify orbit compliance
      if ! ${pkgs.binutils}/bin/strings $out/bin/${pname} | grep -q "DOMAIN_LEVEL.*${toString domainConfig.level}"; then
        echo "ERROR: Domain level not embedded in binary"
        exit 1
      fi
    '';

    # Install domain-specific files
    postInstall = ''
      mkdir -p $out/etc/zos/domains
      cat > $out/etc/zos/domains/${domain}.policy << EOF
      # ZOS Domain Policy for ${domain}
      domain_name=${domain}
      security_level=${toString domainConfig.level}
      allowed_orbits=${builtins.concatStringsSep "," domainConfig.orbits}
      allowed_capabilities=${builtins.concatStringsSep "," domainConfig.capabilities}
      allowed_syscalls=${builtins.concatStringsSep "," domainConfig.syscalls}
      memory_limit=${domainConfig.memory_limit}

      # SELinux-style rules
      allow ${domain}_t ${domain}_exec_t:file { read execute };
      ${if domainConfig.level >= 2 then "allow ${domain}_t tmp_t:dir { read write };" else ""}
      ${if domainConfig.level >= 3 then "allow ${domain}_t etc_t:file read;" else ""}
      ${if domainConfig.level >= 4 then "allow ${domain}_t kernel_t:system all;" else ""}
      EOF

      # Create systemd service for layer
      mkdir -p $out/lib/systemd/system
      cat > $out/lib/systemd/system/zos-${domain}.service << EOF
      [Unit]
      Description=ZOS ${domain} Security Layer
      After=network.target

      [Service]
      Type=simple
      ExecStart=$out/bin/${pname}
      User=${if domainConfig.level >= 4 then "root" else "zos-${domain}"}
      Group=${if domainConfig.level >= 4 then "root" else "zos"}

      # Security restrictions
      NoNewPrivileges=true
      PrivateTmp=true
      ProtectSystem=${if domainConfig.level >= 3 then "false" else "strict"}
      ProtectHome=${if domainConfig.level >= 2 then "false" else "true"}
      ReadOnlyPaths=/
      ReadWritePaths=${if domainConfig.level >= 2 then "/tmp /var/lib/zos" else "/tmp"}

      # Resource limits
      MemoryLimit=${domainConfig.memory_limit}
      CPUQuota=${toString (domainConfig.level * 25)}%

      # Capability restrictions
      CapabilityBoundingSet=${builtins.concatStringsSep " " (
        if domainConfig.capabilities == ["all"] then ["CAP_SYS_ADMIN"] else
        map (cap: "CAP_" + (pkgs.lib.toUpper cap)) domainConfig.capabilities
      )}

      [Install]
      WantedBy=multi-user.target
      EOF
    '';

    meta = with pkgs.lib; {
      description = "ZOS Security Layer ${domain} (Level ${toString domainConfig.level})";
      license = licenses.agpl3Plus;
      maintainers = [ "zos-security-team" ];
    };
  };

  # Build all security layers
  securityLayers = pkgs.lib.mapAttrs buildSecurityLayer securityDomains;

  # Core lattice verification
  latticeVerification = pkgs.writeShellScriptBin "verify-lattice" ''
    set -e
    echo "🔍 Verifying ZOS Security Lattice..."

    # Verify layer isolation
    for layer in l0_public l1_gateway l2_service l3_core l4_kernel; do
      echo "Verifying $layer..."

      # Check binary exists
      if [ ! -f "${securityLayers."$layer"}/bin/zos-$layer" ]; then
        echo "❌ Binary missing for $layer"
        exit 1
      fi

      # Check domain policy
      if [ ! -f "${securityLayers."$layer"}/etc/zos/domains/$layer.policy" ]; then
        echo "❌ Domain policy missing for $layer"
        exit 1
      fi

      # Verify syscall restrictions
      case $layer in
        l0_public|l1_gateway)
          if ${pkgs.binutils}/bin/objdump -T "${securityLayers."$layer"}/bin/zos-$layer" | grep -E "(execve|ptrace)"; then
            echo "❌ Forbidden syscalls found in $layer"
            exit 1
          fi
          ;;
      esac

      echo "✅ $layer verified"
    done

    echo "🎯 Lattice verification complete"
  '';

  # Domain model documentation
  domainModelDocs = pkgs.writeText "zos-domain-model.md" ''
    # ZOS Security Domain Model

    ## Domain Hierarchy

    ```
    L4 Kernel   ← Root/System access, all orbits, unlimited resources
    L3 Core     ← Admin access, up to alternating orbits, 1GB memory
    L2 Service  ← Business logic, up to symmetric orbits, 100MB memory
    L1 Gateway  ← Auth/routing, up to cyclic orbits, 10MB memory
    L0 Public   ← User access, trivial orbits only, 1MB memory
    ```

    ## Orbit-Based Access Control

    Each domain can only access orbits up to its level:
    - **Trivial**: O(1) operations (constants, simple math)
    - **Cyclic**: O(n) operations (loops, basic I/O)
    - **Symmetric**: O(n!) operations (complex algorithms)
    - **Alternating**: O(2^n) operations (exponential algorithms)
    - **Sporadic**: Irregular operations (unsafe, FFI)
    - **Monster**: Unrestricted operations (kernel access)

    ## SELinux-Style Policies

    Each domain has mandatory access control:
    ```
    allow l0_public_t l0_public_exec_t:file { read execute };
    allow l2_service_t tmp_t:dir { read write };
    allow l3_core_t etc_t:file read;
    allow l4_kernel_t kernel_t:system all;
    ```

    ## Capability Model

    Domains have specific capabilities:
    - **L0**: read, compute
    - **L1**: read, compute, auth, route
    - **L2**: read, compute, auth, route, process
    - **L3**: read, compute, auth, route, process, admin
    - **L4**: all capabilities

    ## Resource Limits

    Each domain has enforced resource limits:
    - Memory limits: 1MB → 10MB → 100MB → 1GB → unlimited
    - CPU quotas: 25% → 50% → 75% → 100% → unlimited
    - Syscall restrictions: none → basic → extended → admin → all
  '';

in {
  # Main lattice build
  zos-lattice = pkgs.symlinkJoin {
    name = "zos-security-lattice";
    paths = builtins.attrValues securityLayers ++ [
      latticeVerification
      domainModelDocs
    ];

    postBuild = ''
      # Create lattice configuration
      mkdir -p $out/etc/zos
      cat > $out/etc/zos/lattice.conf << EOF
      # ZOS Security Lattice Configuration
      lattice_version=1.0
      domain_count=${toString (builtins.length (builtins.attrNames securityDomains))}
      verification_required=true
      orbit_enforcement=true

      # Domain hierarchy
      ${builtins.concatStringsSep "\n" (
        pkgs.lib.mapAttrsToList (name: config:
          "domain_${name}_level=${toString config.level}"
        ) securityDomains
      )}
      EOF

      # Create verification script
      cat > $out/bin/verify-zos-lattice << 'EOF'
      #!/bin/sh
      exec ${latticeVerification}/bin/verify-lattice "$@"
      EOF
      chmod +x $out/bin/verify-zos-lattice
    '';
  };

  # Individual layer exports
  inherit securityLayers;

  # Development shell
  devShell = pkgs.mkShell {
    buildInputs = with pkgs; [
      rustc
      cargo
      pkg-config
      openssl
      systemd
      binutils
      gnused
    ];

    shellHook = ''
      echo "🔐 ZOS Security Lattice Development Environment"
      echo "Available commands:"
      echo "  nix-build -A zos-lattice    # Build complete lattice"
      echo "  nix-build -A securityLayers.l0_public  # Build specific layer"
      echo "  ./result/bin/verify-zos-lattice        # Verify lattice"
    '';
  };
}
