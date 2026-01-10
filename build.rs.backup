// ZOS Server build.rs - Universal Context Expansion
// AGPL-3.0 License

use zos_build_macros::mkbuild;

mkbuild! {
    // Universal macro definitions that expand to multiple contexts

    contexts: {
        cargo: "feature_lattice",
        bash: "shell_scripts",
        nix: "nix_expressions",
        docker: "containerization",
        systemd: "service_management",
        python: "automation_scripts",
        powershell: "windows_support"
    },

    // Layer -4: Advanced ZK (Rollups, Lattice Folding, HME, MetaCoq, Lean4)
    advanced_zk: {
        rollups: ["bash: zkrollup-build.sh", "nix: zkrollup.nix", "cargo: rollup-features"],
        lattice_folding: ["python: lattice_fold.py", "cargo: folding-lib"],
        hme: ["nix: hme-deps.nix", "systemd: hme.service"],
        metacoq: ["bash: install-coq.sh", "nix: coq-env.nix"],
        lean4: ["bash: lean4-setup.sh", "docker: lean4.dockerfile"]
    },

    // Layer -3: Zero Knowledge (ZK-SNARKs, ZK-STARKs, Correctness Proofs)
    zk: {
        zk_snarks: ["cargo: snark-features", "bash: snark-build.sh"],
        zk_starks: ["cargo: stark-features", "nix: stark-deps.nix"],
        correctness_proofs: ["python: proof-gen.py", "systemd: proof-verifier.service"]
    },

    // Layer -2: Regulatory (SEC, Quality, GDPR/HIPAA/SOX/ISO)
    regulatory: {
        sec_compliance: ["bash: sec-audit.sh", "python: compliance-check.py"],
        gdpr: ["cargo: gdpr-features", "systemd: gdpr-monitor.service"],
        hipaa: ["docker: hipaa-secure.dockerfile", "bash: hipaa-setup.sh"],
        sox: ["python: sox-audit.py", "nix: sox-env.nix"],
        iso: ["bash: iso-cert.sh", "systemd: iso-compliance.service"]
    },

    // Layer -1: Governance (Voting, Resources, ERP)
    governance: {
        voting_systems: ["python: voting.py", "bash: vote-setup.sh"],
        resource_management: ["systemd: resource-manager.service", "bash: resource-monitor.sh"],
        erp_integration: ["docker: erp-connector.dockerfile", "python: erp-sync.py"]
    },

    // Layer 0: Foundation (LMFDB, Wikidata, OSM, Archive.org, SDF.org)
    foundation: {
        lmfdb: ["python: lmfdb-client.py", "bash: lmfdb-sync.sh", "nix: lmfdb-env.nix"],
        wikidata: ["python: wikidata-extract.py", "bash: wiki-update.sh"],
        openstreetmap: ["bash: osm-download.sh", "docker: osm-server.dockerfile"],
        archive_org: ["python: archive-backup.py", "systemd: archive-sync.service"],
        sdf_org: ["bash: sdf-connect.sh", "nix: sdf-client.nix"]
    },

    // Layer 1: System (19 plugins)
    system: {
        systemd: ["systemd: zos-core.service", "bash: systemd-setup.sh"],
        docker: ["docker: zos-runtime.dockerfile", "bash: docker-build.sh"],
        compilers: ["nix: compiler-toolchain.nix", "bash: compiler-setup.sh"],
        blockchain: ["python: blockchain-sync.py", "docker: blockchain-node.dockerfile"],
        kernel: ["bash: kernel-module.sh", "nix: kernel-dev.nix"],
        ebpf: ["bash: ebpf-compile.sh", "python: ebpf-loader.py"],
        wasm: ["cargo: wasm-features", "bash: wasm-build.sh"],
        runtime_plugins: ["systemd: runtime.service", "python: plugin-manager.py"],
        storage: ["bash: storage-setup.sh", "docker: storage-cluster.dockerfile"],
        networking: ["systemd: network-manager.service", "bash: net-config.sh"],
        security: ["python: security-scan.py", "bash: security-hardening.sh"],
        telemetry: ["systemd: telemetry.service", "python: metrics-collector.py"],
        debug: ["bash: debug-tools.sh", "python: debug-analyzer.py"],
        bintools: ["nix: bintools.nix", "bash: bintools-install.sh"],
        libp2p: ["cargo: p2p-features", "docker: p2p-node.dockerfile"],
        solana: ["bash: solana-setup.sh", "python: solana-client.py"],
        oracle: ["systemd: oracle.service", "python: oracle-connector.py"],
        enterprise: ["docker: enterprise.dockerfile", "powershell: enterprise-setup.ps1"],
        modeling: ["python: model-trainer.py", "nix: ml-env.nix"]
    },

    // Layer 2: Data Formats (Parquet, HuggingFace, RDF, SQL, Protocols)
    data_formats: {
        parquet: ["python: parquet-processor.py", "bash: parquet-tools.sh"],
        huggingface: ["python: hf-integration.py", "docker: hf-models.dockerfile"],
        rdf: ["python: rdf-parser.py", "bash: rdf-validate.sh"],
        sql: ["bash: db-setup.sh", "python: sql-migrator.py"],
        protocols: ["python: protocol-handler.py", "systemd: protocol-server.service"],
        dataflow: ["python: dataflow-engine.py", "docker: dataflow.dockerfile"],
        knowledge: ["python: knowledge-graph.py", "bash: kg-build.sh"]
    },

    // Layer ∞: Recursive (Each layer exports to all others infinitely)
    recursive: {
        infinite_export: ["python: recursive-export.py", "bash: layer-sync.sh"],
        cross_layer_communication: ["systemd: layer-bridge.service", "python: layer-comm.py"],
        recursive_verification: ["python: recursive-verify.py", "bash: verify-all.sh"]
    }
}
