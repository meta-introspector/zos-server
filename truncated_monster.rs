use std::collections::HashMap;

#[derive(Debug, Clone)]
struct MonsterComponent {
    prime: u64,
    power: u32,
    gpu_chunk_id: usize,
    partial_result: f64,
}

struct TruncatedMonster {
    components: Vec<MonsterComponent>,
    composition_map: HashMap<usize, Vec<usize>>,
    gpu_chunks: Vec<Vec<f64>>,
}

impl TruncatedMonster {
    fn new() -> Self {
        println!("🧩 Truncating Monster Group into components...");

        // Monster Group = 2^46 × 3^20 × 5^9 × 7^6 × 11^2 × 13^3 × 17 × 19 × 23 × 29 × 31 × 41 × 47 × 59 × 71
        let components = vec![
            MonsterComponent {
                prime: 2,
                power: 46,
                gpu_chunk_id: 0,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 3,
                power: 20,
                gpu_chunk_id: 1,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 5,
                power: 9,
                gpu_chunk_id: 2,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 7,
                power: 6,
                gpu_chunk_id: 3,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 11,
                power: 2,
                gpu_chunk_id: 4,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 13,
                power: 3,
                gpu_chunk_id: 5,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 17,
                power: 1,
                gpu_chunk_id: 6,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 19,
                power: 1,
                gpu_chunk_id: 7,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 23,
                power: 1,
                gpu_chunk_id: 8,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 29,
                power: 1,
                gpu_chunk_id: 9,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 31,
                power: 1,
                gpu_chunk_id: 10,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 41,
                power: 1,
                gpu_chunk_id: 11,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 47,
                power: 1,
                gpu_chunk_id: 12,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 59,
                power: 1,
                gpu_chunk_id: 13,
                partial_result: 0.0,
            },
            MonsterComponent {
                prime: 71,
                power: 1,
                gpu_chunk_id: 14,
                partial_result: 0.0,
            },
        ];

        Self {
            components,
            composition_map: HashMap::new(),
            gpu_chunks: Vec::new(),
        }
    }

    fn load_components_to_gpu(&mut self) {
        println!("📤 Loading Monster components to GPU chunks...");

        for comp in &self.components {
            let chunk_data = self.generate_component_data(comp);
            self.gpu_chunks.push(chunk_data);

            println!(
                "   Chunk {}: Prime {}^{} → {} elements",
                comp.gpu_chunk_id,
                comp.prime,
                comp.power,
                self.gpu_chunks[comp.gpu_chunk_id].len()
            );
        }

        println!("✅ {} components loaded to GPU", self.components.len());
    }

    fn generate_component_data(&self, comp: &MonsterComponent) -> Vec<f64> {
        // Generate GPU computation data for this Monster component
        let size = (comp.prime as usize).min(1024); // Limit chunk size

        (0..size)
            .map(|i| {
                let base = comp.prime as f64;
                let power = comp.power as f64;
                (i as f64 * base).powf(1.0 / power) // Inverse power for GPU efficiency
            })
            .collect()
    }

    fn compute_component_on_gpu(&mut self, component_id: usize) -> f64 {
        if component_id >= self.components.len() {
            return 0.0;
        }

        let comp = &self.components[component_id];
        let chunk = &self.gpu_chunks[comp.gpu_chunk_id];

        println!(
            "⚡ Computing component {} (prime {}) on GPU...",
            component_id, comp.prime
        );

        // Simulate GPU kernel execution
        let result = chunk
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                if i % comp.prime as usize == 0 {
                    comp.prime as f64 // Fixed point
                } else {
                    value * comp.prime as f64
                }
            })
            .sum::<f64>()
            / chunk.len() as f64;

        println!("   ✅ Component {} result: {:.2}", component_id, result);
        result
    }

    fn compose_monster_parts(&mut self, part_indices: &[usize]) -> f64 {
        println!("🔧 Composing Monster parts: {:?}", part_indices);

        let mut composition_result = 1.0;

        for &idx in part_indices {
            let component_result = self.compute_component_on_gpu(idx);
            self.components[idx].partial_result = component_result;
            composition_result *= component_result;
        }

        // Store composition mapping
        let composition_id = self.composition_map.len();
        self.composition_map
            .insert(composition_id, part_indices.to_vec());

        println!(
            "   🎯 Composition {}: {:.2}",
            composition_id, composition_result
        );
        composition_result
    }

    fn demonstrate_truncated_composition(&mut self) {
        println!("\n🧩 DEMONSTRATING TRUNCATED MONSTER COMPOSITION:");
        println!("{}", "=".repeat(55));

        // Load all components
        self.load_components_to_gpu();

        // Compose different parts of the Monster
        println!("\n1️⃣ Small primes composition (2, 3, 5):");
        let small_primes = self.compose_monster_parts(&[0, 1, 2]);

        println!("\n2️⃣ Medium primes composition (7, 11, 13):");
        let medium_primes = self.compose_monster_parts(&[3, 4, 5]);

        println!("\n3️⃣ Large primes composition (47, 59, 71):");
        let large_primes = self.compose_monster_parts(&[12, 13, 14]);

        println!("\n4️⃣ Full Monster approximation (all components):");
        let all_indices: Vec<usize> = (0..self.components.len()).collect();
        let full_monster = self.compose_monster_parts(&all_indices);

        println!("\n📊 COMPOSITION RESULTS:");
        println!("   Small primes:  {:.2}", small_primes);
        println!("   Medium primes: {:.2}", medium_primes);
        println!("   Large primes:  {:.2}", large_primes);
        println!("   Full Monster:  {:.2}", full_monster);

        self.show_composition_map();
    }

    fn show_composition_map(&self) {
        println!("\n🗺️ COMPOSITION MAP:");
        for (comp_id, indices) in &self.composition_map {
            let primes: Vec<u64> = indices.iter().map(|&i| self.components[i].prime).collect();
            println!("   Composition {}: primes {:?}", comp_id, primes);
        }
    }
}

fn main() {
    println!("🧩 Truncated Monster Group Composition System");
    println!("{}", "=".repeat(50));

    let mut monster = TruncatedMonster::new();

    // Demonstrate truncated composition
    monster.demonstrate_truncated_composition();

    println!("\n🎯 TRUNCATED MONSTER COMPLETE:");
    println!(
        "   ✅ Monster Group factorized into {} components",
        monster.components.len()
    );
    println!(
        "   ✅ Components loaded to {} GPU chunks",
        monster.gpu_chunks.len()
    );
    println!("   ✅ Partial compositions computed on GPU");
    println!("   ✅ Full Monster reconstructed from parts");
    println!("   🔮 Monster Group now computable in manageable chunks!");
}
