//! seal_plugin.rs — ZOS plugin: paid geometric seal generation service.
//!
//! Commands:
//!   seal generate <dna_json> <state_json>  — generate sealed PNG (costs 1 token)
//!   seal verify <png_path>                 — verify seal integrity (free)
//!   seal price                             — show current pricing
//!   seal stats                             — generation stats
//!
//! The seal encoding algorithm is PRIVATE. Clients pay to generate,
//! verify is free (encourages adoption).

use crate::traits::ZOSPlugin;
use async_trait::async_trait;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};

static SEALS_GENERATED: AtomicU64 = AtomicU64::new(0);
static REVENUE_TOKENS: AtomicU64 = AtomicU64::new(0);

pub struct SealPlugin;

// Pricing (in SOLFUNMEME tokens)
const PRICE_GENERATE: u64 = 1;
const PRICE_CUSTOM: u64 = 5;
const PRICE_BATCH: u64 = 0; // 0.5 per, handled as 1 per 2

#[async_trait]
impl ZOSPlugin for SealPlugin {
    fn name(&self) -> &'static str { "seal" }
    fn version(&self) -> &'static str { "0.1.0" }

    fn commands(&self) -> Vec<&'static str> {
        vec!["seal-generate", "seal-verify", "seal-price", "seal-stats"]
    }

    async fn execute(&self, command: &str, args: Vec<String>) -> Result<Value, String> {
        match command {
            "seal-generate" => self.generate(args).await,
            "seal-verify" => self.verify(args).await,
            "seal-price" => Ok(json!({
                "generate": PRICE_GENERATE,
                "custom_embedding": PRICE_CUSTOM,
                "batch_per_2": 1,
                "verify": "free",
                "currency": "SOLFUNMEME"
            })),
            "seal-stats" => Ok(json!({
                "seals_generated": SEALS_GENERATED.load(Ordering::Relaxed),
                "revenue_tokens": REVENUE_TOKENS.load(Ordering::Relaxed),
                "capacity_bytes": 196_572,
                "torus_cells": 196_883,
                "primes": [71, 59, 47],
            })),
            _ => Err(format!("unknown command: {command}")),
        }
    }
}

impl SealPlugin {
    async fn generate(&self, args: Vec<String>) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("usage: seal generate <dna_json> <state_json>".into());
        }

        let dna: Vec<Vec<u64>> = serde_json::from_str(&args[0])
            .map_err(|e| format!("invalid dna: {e}"))?;
        let state: Vec<f32> = serde_json::from_str(&args[1])
            .map_err(|e| format!("invalid state: {e}"))?;

        if state.len() != 24 {
            return Err("state must be 24 floats (Leech lattice Λ₂₄)".into());
        }

        // Pack: SEAL header + 24D state + DNA + no WASM
        let dna_bytes = serde_json::to_vec(&dna).unwrap();
        let packed = pack_seal(&state, &dna_bytes);

        // Encode into 512×512 RGB
        let rgb = encode_seal(&packed)
            .map_err(|e| format!("seal encode failed: {e}"))?;

        // Compute orbifold + hash
        let hash = hex::encode(Sha256::digest(&packed));
        let orbifold = orbifold_coords(&dna_bytes);

        SEALS_GENERATED.fetch_add(1, Ordering::Relaxed);
        REVENUE_TOKENS.fetch_add(PRICE_GENERATE, Ordering::Relaxed);

        Ok(json!({
            "status": "sealed",
            "hash": hash,
            "orbifold": orbifold,
            "rgb_len": rgb.len(),
            "cost": PRICE_GENERATE,
            "seal_id": SEALS_GENERATED.load(Ordering::Relaxed),
        }))
    }

    async fn verify(&self, args: Vec<String>) -> Result<Value, String> {
        if args.is_empty() {
            return Err("usage: seal verify <rgb_hex_or_path>".into());
        }

        let rgb = hex::decode(&args[0]).map_err(|e| format!("invalid hex: {e}"))?;
        match decode_seal(&rgb) {
            Ok(payload) => {
                let hash = hex::encode(Sha256::digest(&payload));
                Ok(json!({ "valid": true, "hash": hash, "payload_len": payload.len() }))
            }
            Err(e) => Ok(json!({ "valid": false, "error": e })),
        }
    }
}

// ── Private seal implementation (same as erdfa-publish/src/seal.rs) ──

const SP: usize = 512 * 512;
const SL: usize = 6;
const SC: usize = SP * SL / 8;
const SH: usize = 36;

fn bp_embed(rgb: &mut [u8], data: &[u8]) {
    for (i, &byte) in data.iter().enumerate() {
        if i >= SC { break; }
        for b in 0..8u8 {
            let bi = i * 8 + b as usize;
            let (px, pl) = (bi / SL, bi % SL);
            if px >= SP { return; }
            let idx = px * 3 + pl % 3;
            let bp = pl / 3;
            rgb[idx] = (rgb[idx] & !(1 << bp)) | (((byte >> b) & 1) << bp);
        }
    }
}

fn bp_extract(rgb: &[u8], len: usize) -> Vec<u8> {
    (0..len.min(SC)).map(|i| (0..8u8).map(|b| {
        let bi = i * 8 + b as usize;
        let (px, pl) = (bi / SL, bi % SL);
        if px >= SP { return 0; }
        ((rgb[px * 3 + pl % 3] >> (pl / 3)) & 1) << b
    }).sum()).collect()
}

fn pack_seal(state: &[f32], dna: &[u8]) -> Vec<u8> {
    let mut buf = b"SEAL".to_vec();
    for f in state { buf.extend_from_slice(&f.to_le_bytes()); }
    buf.extend_from_slice(&(dna.len() as u32).to_le_bytes());
    buf.extend_from_slice(dna);
    buf.extend_from_slice(&0u32.to_le_bytes()); // no wasm
    buf
}

fn encode_seal(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() + SH > SC { return Err("payload too large".into()); }
    let h = Sha256::digest(data);
    let mut w = Vec::with_capacity(SH + data.len());
    w.extend_from_slice(&(data.len() as u32).to_le_bytes());
    w.extend_from_slice(&h);
    w.extend_from_slice(data);
    let mut rgb = vec![128u8; SP * 3];
    bp_embed(&mut rgb, &w);
    Ok(rgb)
}

fn decode_seal(rgb: &[u8]) -> Result<Vec<u8>, String> {
    if rgb.len() < SP * 3 { return Err("buffer too small".into()); }
    let hdr = bp_extract(rgb, SH);
    let len = u32::from_le_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]) as usize;
    if len == 0 || len + SH > SC { return Err("invalid seal".into()); }
    let w = bp_extract(rgb, SH + len);
    let payload = &w[36..36 + len];
    let stored: [u8; 32] = w[4..36].try_into().map_err(|_| "hash error")?;
    if stored != <[u8; 32]>::from(Sha256::digest(payload)) {
        return Err("integrity check failed".into());
    }
    Ok(payload.to_vec())
}

fn orbifold_coords(data: &[u8]) -> (u8, u8, u8) {
    let h = Sha256::digest(data);
    let v = u64::from_le_bytes(h[0..8].try_into().unwrap());
    ((v % 71) as u8, (v % 59) as u8, (v % 47) as u8)
}
