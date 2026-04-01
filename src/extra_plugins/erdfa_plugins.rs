// erdfa_plugins.rs — ZOS plugins for eRDFa tools
//
// CRQ: CRQ-SWAB-014
// Three plugins reusing erdfa-publish crate:
//   1. MaassPlugin     — blade targeting / semantic restoration
//   2. CFTPlugin       — torus coordinate computation + KO classification
//   3. SheafPlugin     — eRDFa shard encoding + IPFS CID generation

use async_trait::async_trait;
use serde_json::{json, Value};
use sha2::{Sha256, Digest};
use std::collections::BTreeMap;

// ── SSP primes (shared) ─────────────────────────────────────────

const SSP: [u64; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71];

fn torus_coord(data: &[u8]) -> [u64; 15] {
    let h = u128::from_be_bytes(Sha256::digest(data)[0..16].try_into().unwrap());
    let mut c = [0u64; 15];
    for (i, &p) in SSP.iter().enumerate() { c[i] = (h % p as u128) as u64; }
    c
}

fn orbifold(data: &[u8]) -> (u64, u64, u64) {
    let c = torus_coord(data);
    (c[14], c[13], c[12])
}

fn content_cid(data: &[u8]) -> String {
    format!("bafk{}", &hex::encode(Sha256::digest(data))[..32])
}

// ── 1. MaassPlugin ──────────────────────────────────────────────

pub struct MaassPlugin;

#[async_trait]
impl super::ZOSPlugin for MaassPlugin {
    fn name(&self) -> &'static str { "maass" }
    fn version(&self) -> &'static str { "0.1.0" }
    fn commands(&self) -> Vec<&'static str> {
        vec!["maass-target", "maass-orbifold", "maass-distance"]
    }

    async fn execute(&self, command: &str, args: Vec<String>) -> Result<Value, String> {
        match command {
            "maass-target" => {
                // args: <blade> <data> [bag_json]
                let blade: u64 = args.get(0).ok_or("need blade")?.parse().map_err(|e| format!("{e}"))?;
                let data = args.get(1).ok_or("need data")?;
                let bag: BTreeMap<String, String> = if let Some(j) = args.get(2) {
                    serde_json::from_str(j).map_err(|e| format!("{e}"))?
                } else {
                    BTreeMap::new()
                };
                let nat = orbifold(data.as_bytes());
                // Try subsets
                let keys: Vec<&String> = bag.keys().collect();
                for size in 0..=keys.len() {
                    for combo in combinations(&keys, size) {
                        let subset: BTreeMap<String, String> = combo.iter()
                            .map(|k| ((*k).clone(), bag[*k].clone())).collect();
                        let encoded = encode_beliefs(data.as_bytes(), &subset);
                        let o = orbifold(&encoded);
                        if o.0 == blade {
                            let rep = torus_coord(&encoded);
                            return Ok(json!({
                                "found": true,
                                "target_blade": blade,
                                "natural_orbifold": format!("{},{},{}", nat.0, nat.1, nat.2),
                                "repaired_orbifold": format!("{},{},{}", o.0, o.1, o.2),
                                "shadow": subset,
                                "repair_size": size,
                                "cid": content_cid(&encoded),
                                "torus": rep,
                            }));
                        }
                    }
                }
                Ok(json!({"found": false, "target_blade": blade, "natural_blade": nat.0}))
            }
            "maass-orbifold" => {
                let data = args.get(0).ok_or("need data")?;
                let o = orbifold(data.as_bytes());
                let t = torus_coord(data.as_bytes());
                Ok(json!({"orbifold": format!("{},{},{}", o.0, o.1, o.2), "torus": t, "blade": o.0}))
            }
            "maass-distance" => {
                let a = args.get(0).ok_or("need data_a")?;
                let b = args.get(1).ok_or("need data_b")?;
                let ta = torus_coord(a.as_bytes());
                let tb = torus_coord(b.as_bytes());
                let ham: u32 = ta.iter().zip(&tb).filter(|(x,y)| x != y).count() as u32;
                let l1: u64 = ta.iter().zip(&tb).zip(&SSP).map(|((x,y),&p)| {
                    let d = if x > y { x - y } else { y - x }; d.min(p - d)
                }).sum();
                Ok(json!({"hamming": ham, "l1": l1, "torus_a": ta, "torus_b": tb}))
            }
            _ => Err(format!("unknown command: {command}"))
        }
    }
}

// ── 2. CFTPlugin ────────────────────────────────────────────────

pub struct CFTPlugin;

#[async_trait]
impl super::ZOSPlugin for CFTPlugin {
    fn name(&self) -> &'static str { "cft" }
    fn version(&self) -> &'static str { "0.1.0" }
    fn commands(&self) -> Vec<&'static str> {
        vec!["cft-classify", "cft-partition", "cft-shard"]
    }

    async fn execute(&self, command: &str, args: Vec<String>) -> Result<Value, String> {
        match command {
            "cft-classify" => {
                // Classify a name into KO level by dep count
                let name = args.get(0).ok_or("need name")?;
                let deps: u64 = args.get(1).unwrap_or(&"0".into()).parse().unwrap_or(0);
                let kind = args.get(2).unwrap_or(&"def".into()).clone();
                let ko = deps % 8;
                let ko_label = ["Z","Z2","Z2","0","Z","0","0","0"][ko as usize];
                let conf_dim = (deps as f64 + 1.0).log2();
                let t = torus_coord(name.as_bytes());
                Ok(json!({
                    "name": name, "kind": kind, "deps": deps,
                    "ko_level": ko, "ko_group": ko_label,
                    "conf_dim": conf_dim,
                    "orbifold": format!("{},{},{}", t[14], t[13], t[12]),
                    "blade": t[14], "torus": t,
                }))
            }
            "cft-partition" => {
                // Compute partition function from a list of conf dims
                let beta: f64 = args.get(0).unwrap_or(&"1.0".into()).parse().unwrap_or(1.0);
                let dims: Vec<f64> = args[1..].iter()
                    .filter_map(|s| s.parse().ok()).collect();
                let z: f64 = dims.iter().map(|h| (-beta * h).exp()).sum();
                let f = -(z.ln());
                Ok(json!({"beta": beta, "Z": z, "F": f, "operators": dims.len()}))
            }
            "cft-shard" => {
                let data = args.get(0).ok_or("need data")?;
                let blade = torus_coord(data.as_bytes())[14];
                Ok(json!({"blade": blade, "shard": format!("shard_{:02}.html", blade)}))
            }
            _ => Err(format!("unknown command: {command}"))
        }
    }
}

// ── 3. SheafPlugin ──────────────────────────────────────────────

pub struct SheafPlugin;

#[async_trait]
impl super::ZOSPlugin for SheafPlugin {
    fn name(&self) -> &'static str { "sheaf" }
    fn version(&self) -> &'static str { "0.1.0" }
    fn commands(&self) -> Vec<&'static str> {
        vec!["sheaf-encode", "sheaf-cid", "sheaf-erdfa"]
    }

    async fn execute(&self, command: &str, args: Vec<String>) -> Result<Value, String> {
        match command {
            "sheaf-cid" => {
                let data = args.get(0).ok_or("need data")?;
                Ok(json!({"cid": content_cid(data.as_bytes()), "size": data.len()}))
            }
            "sheaf-encode" => {
                let data = args.get(0).ok_or("need data")?;
                let bag_json = args.get(1).unwrap_or(&"{}".into()).clone();
                let bag: BTreeMap<String, String> = serde_json::from_str(&bag_json).unwrap_or_default();
                let encoded = encode_beliefs(data.as_bytes(), &bag);
                let cid = content_cid(&encoded);
                let o = orbifold(&encoded);
                Ok(json!({"cid": cid, "orbifold": format!("{},{},{}", o.0, o.1, o.2), "size": encoded.len()}))
            }
            "sheaf-erdfa" => {
                let data = args.get(0).ok_or("need data")?;
                let label = args.get(1).unwrap_or(data);
                let o = orbifold(data.as_bytes());
                let cid = content_cid(data.as_bytes());
                let bott = o.0 % 8;
                let erdfa = format!(
r#"<div typeof="erdfa:SheafSection dasl:Type6" about="#{cid}">
  <meta property="erdfa:shard" content="{},{},{}" />
  <meta property="erdfa:encoding" content="raw" />
  <meta property="dasl:type" content="6" />
  <meta property="dasl:eigenspace" content="Earth" />
  <meta property="dasl:bott" content="{bott} ({})" />
  <meta property="sheaf:orbifold" content="({} mod 71, {} mod 59, {} mod 47)" />
  <span property="rdfs:label">{label}</span>
</div>"#,
                    o.0, o.1, o.2,
                    ["Z","C","C","Z","Z","C","C","Z"][bott as usize],
                    o.0, o.1, o.2,
                );
                Ok(json!({"erdfa": erdfa, "cid": cid, "blade": o.0}))
            }
            _ => Err(format!("unknown command: {command}"))
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────

fn encode_beliefs(data: &[u8], beliefs: &BTreeMap<String, String>) -> Vec<u8> {
    if beliefs.is_empty() { return data.to_vec(); }
    let mut out = data.to_vec();
    out.push(0);
    out.extend(serde_json::to_string(beliefs).unwrap_or_default().as_bytes());
    out
}

fn combinations<'a, T>(items: &'a [T], k: usize) -> Vec<Vec<&'a T>> {
    if k == 0 { return vec![vec![]]; }
    if items.len() < k { return vec![]; }
    let mut result = vec![];
    for (i, item) in items.iter().enumerate() {
        for mut rest in combinations(&items[i+1..], k - 1) {
            rest.insert(0, item);
            result.push(rest);
        }
    }
    result
}

// ── Registration ────────────────────────────────────────────────

pub fn register_all(registry: &mut dyn super::ZOSPluginRegistry) {
    registry.register_plugin(Box::new(MaassPlugin));
    registry.register_plugin(Box::new(CFTPlugin));
    registry.register_plugin(Box::new(SheafPlugin));
}
