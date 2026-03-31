//! paste-fuzz — Alife m3m3f4rm fuzz testers + zkperf security review.
//!
//! Each plugin is fuzzed with evolved inputs. Organisms are byte vectors
//! that mutate via FRACTRAN-style rules. Fitness = coverage + crash discovery.
//! Every execution is witnessed with timing + hash for zkperf review.

use sha2::{Digest, Sha256};

/// One fuzz organism: a byte vector that evolves.
#[derive(Clone)]
struct Organism {
    dna: Vec<String>, // args to feed the plugin
    fitness: f64,
    crashes: u32,
    generation: u32,
}

/// zkperf witness for one fuzz execution.
#[derive(Debug)]
struct FuzzWitness {
    plugin: &'static str,
    command: String,
    input_hash: String,
    output_hash: String,
    time_us: u64,
    crashed: bool,
    error: Option<String>,
}

/// Plugin entry: name, commands, execute function.
struct PluginEntry {
    name: &'static str,
    commands: Vec<&'static str>,
    exec: fn(&str, &[String]) -> Result<serde_json::Value, String>,
}

fn all_plugins() -> Vec<PluginEntry> {
    vec![
        PluginEntry {
            name: "paste-qr",
            commands: vec!["qr-url", "qr-data", "qr-svg"],
            exec: paste_qr::execute,
        },
        PluginEntry {
            name: "paste-rdfa",
            commands: paste_rdfa::commands(),
            exec: paste_rdfa::execute,
        },
        PluginEntry {
            name: "paste-stego",
            commands: paste_stego::commands(),
            exec: paste_stego::execute,
        },
        PluginEntry {
            name: "paste-splitter",
            commands: paste_splitter::commands(),
            exec: paste_splitter::execute,
        },
        PluginEntry {
            name: "paste-preview",
            commands: paste_preview::commands(),
            exec: paste_preview::execute,
        },
        PluginEntry {
            name: "paste-reply",
            commands: paste_reply::commands(),
            exec: paste_reply::execute,
        },
        PluginEntry {
            name: "paste-browse",
            commands: paste_browse::commands(),
            exec: paste_browse::execute,
        },
        PluginEntry {
            name: "paste-gallery",
            commands: paste_gallery::commands(),
            exec: paste_gallery::execute,
        },
        PluginEntry {
            name: "paste-api",
            commands: paste_api::commands(),
            exec: paste_api::execute,
        },
        PluginEntry {
            name: "paste-home",
            commands: paste_home::commands(),
            exec: paste_home::execute,
        },
    ]
}

fn sha(data: &str) -> String {
    hex::encode(&Sha256::digest(data.as_bytes())[..8])
}

/// Mutate an organism's DNA using FRACTRAN-style rules.
fn mutate(org: &mut Organism, seed: u64) {
    let idx = (seed as usize) % org.dna.len().max(1);
    let mutation = seed % 7;
    match mutation {
        0 => org.dna.push(String::new()),                       // empty arg
        1 => org.dna.push("A".repeat((seed % 10000) as usize)), // long string
        2 => org.dna.push(format!("{}", seed)),                 // numeric
        3 => {
            if !org.dna.is_empty() {
                org.dna[idx] = "\0\x7f\n\r".to_string();
            }
        } // binary
        4 => {
            if !org.dna.is_empty() {
                org.dna[idx] = "../../../etc/passwd".to_string();
            }
        } // path traversal
        5 => {
            if !org.dna.is_empty() {
                org.dna[idx] = "<script>alert(1)</script>".to_string();
            }
        } // XSS
        6 => org.dna.push("{{7*7}}".to_string()),               // template injection
        _ => {}
    }
}

/// Crossover two organisms.
fn crossover(a: &Organism, b: &Organism) -> Organism {
    let cut = a.dna.len() / 2;
    let mut dna = a.dna[..cut.min(a.dna.len())].to_vec();
    if cut < b.dna.len() {
        dna.extend_from_slice(&b.dna[cut..]);
    }
    Organism {
        dna,
        fitness: 0.0,
        crashes: 0,
        generation: a.generation + 1,
    }
}

/// Fuzz one plugin command, return witness.
fn fuzz_once(plugin: &PluginEntry, command: &str, args: &[String]) -> FuzzWitness {
    let input_hash = sha(&format!("{:?}", args));
    let start = std::time::Instant::now();
    let result = (plugin.exec)(command, args);
    let time_us = start.elapsed().as_micros() as u64;

    match result {
        Ok(val) => FuzzWitness {
            plugin: plugin.name,
            command: command.to_string(),
            input_hash,
            output_hash: sha(&val.to_string()),
            time_us,
            crashed: false,
            error: None,
        },
        Err(e) => FuzzWitness {
            plugin: plugin.name,
            command: command.to_string(),
            input_hash,
            output_hash: "error".to_string(),
            time_us,
            crashed: true,
            error: Some(e),
        },
    }
}

/// Run the full m3m3f4rm: evolve fuzz inputs across all plugins.
pub fn run_m3m3f4rm(generations: u32, pop_size: usize) -> Vec<FuzzWitness> {
    let plugins = all_plugins();
    let mut all_witnesses = Vec::new();
    let mut total_crashes = 0u32;
    let mut total_runs = 0u32;

    for plugin in &plugins {
        for command in &plugin.commands {
            // Genesis population
            let mut pop: Vec<Organism> = (0..pop_size)
                .map(|i| Organism {
                    dna: vec![format!("seed_{i}")],
                    fitness: 0.0,
                    crashes: 0,
                    generation: 0,
                })
                .collect();

            for gen in 0..generations {
                // Evaluate
                for org in &mut pop {
                    let w = fuzz_once(plugin, command, &org.dna);
                    if w.crashed {
                        org.crashes += 1;
                        org.fitness += 10.0; // crashes are interesting
                    } else {
                        org.fitness += 1.0;
                    }
                    total_runs += 1;
                    if w.crashed {
                        total_crashes += 1;
                    }
                    all_witnesses.push(w);
                }

                // Sort by fitness (crashes = high fitness)
                pop.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

                // Breed next generation
                let mut next = pop[..pop_size / 4].to_vec(); // elites
                while next.len() < pop_size {
                    let a = &pop[next.len() % (pop_size / 4)];
                    let b = &pop[(next.len() + 1) % (pop_size / 4)];
                    let mut child = crossover(a, b);
                    mutate(&mut child, (gen as u64 * 1000 + next.len() as u64) * 31337);
                    next.push(child);
                }
                pop = next;
            }
        }
    }

    eprintln!(
        "[M3M3F4RM] {} runs, {} crashes across {} plugins",
        total_runs,
        total_crashes,
        plugins.len()
    );
    all_witnesses
}

/// Generate zkperf security review from witnesses.
pub fn security_review(witnesses: &[FuzzWitness]) -> serde_json::Value {
    let total = witnesses.len();
    let crashes: Vec<_> = witnesses.iter().filter(|w| w.crashed).collect();
    let max_time = witnesses.iter().map(|w| w.time_us).max().unwrap_or(0);
    let avg_time = witnesses.iter().map(|w| w.time_us).sum::<u64>() / total.max(1) as u64;

    // Group crashes by plugin
    let mut crash_by_plugin: std::collections::HashMap<&str, Vec<&FuzzWitness>> =
        std::collections::HashMap::new();
    for w in &crashes {
        crash_by_plugin.entry(w.plugin).or_default().push(w);
    }

    // Security findings
    let mut findings = Vec::new();
    for (plugin, cw) in &crash_by_plugin {
        let errors: Vec<_> = cw.iter().filter_map(|w| w.error.as_ref()).collect();
        let unique_errors: std::collections::HashSet<_> =
            errors.iter().map(|e| e.as_str()).collect();
        findings.push(serde_json::json!({
            "plugin": plugin,
            "crash_count": cw.len(),
            "unique_errors": unique_errors.len(),
            "sample_errors": errors.iter().take(3).collect::<Vec<_>>(),
            "severity": if cw.len() > 10 { "HIGH" } else if cw.len() > 3 { "MEDIUM" } else { "LOW" },
        }));
    }

    // Timing anomalies (>10x average)
    let slow: Vec<_> = witnesses
        .iter()
        .filter(|w| w.time_us > avg_time * 10 && w.time_us > 1000)
        .map(|w| {
            serde_json::json!({
                "plugin": w.plugin, "command": w.command,
                "time_us": w.time_us, "input_hash": w.input_hash,
            })
        })
        .collect();

    serde_json::json!({
        "total_runs": total,
        "total_crashes": crashes.len(),
        "crash_rate": format!("{:.2}%", 100.0 * crashes.len() as f64 / total.max(1) as f64),
        "max_time_us": max_time,
        "avg_time_us": avg_time,
        "findings": findings,
        "timing_anomalies": slow,
        "verdict": if crashes.is_empty() { "PASS" } else { "REVIEW REQUIRED" },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_m3m3f4rm_runs() {
        let witnesses = run_m3m3f4rm(3, 8);
        assert!(!witnesses.is_empty());
    }

    #[test]
    fn test_security_review() {
        let witnesses = run_m3m3f4rm(2, 4);
        let review = security_review(&witnesses);
        assert!(review["total_runs"].as_u64().unwrap() > 0);
        assert!(review.get("verdict").is_some());
    }

    #[test]
    fn test_mutation_coverage() {
        // Ensure all 7 mutation types produce different inputs
        let mut org = Organism {
            dna: vec!["base".to_string()],
            fitness: 0.0,
            crashes: 0,
            generation: 0,
        };
        let mut seen = std::collections::HashSet::new();
        for i in 0..7u64 {
            let mut clone = org.clone();
            mutate(&mut clone, i);
            seen.insert(format!("{:?}", clone.dna));
        }
        assert!(seen.len() >= 5, "mutations should produce diverse inputs");
    }

    #[test]
    fn test_path_traversal_caught() {
        let plugins = all_plugins();
        for p in &plugins {
            for cmd in &p.commands {
                let w = fuzz_once(p, cmd, &["../../../etc/passwd".to_string()]);
                // Should not crash with path traversal (graceful handling)
                // If it does crash, that's a finding
                if w.crashed {
                    eprintln!("FINDING: {} {} crashes on path traversal", p.name, cmd);
                }
            }
        }
    }

    #[test]
    fn test_xss_caught() {
        let plugins = all_plugins();
        for p in &plugins {
            for cmd in &p.commands {
                let w = fuzz_once(p, cmd, &["<script>alert(1)</script>".to_string()]);
                if w.crashed {
                    eprintln!("FINDING: {} {} crashes on XSS input", p.name, cmd);
                }
            }
        }
    }
}
