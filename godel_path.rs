use std::collections::HashMap;

struct Kleene2Markov2Godel {
    godel_map: HashMap<String, u64>,
    next_godel: u64,
}

impl Kleene2Markov2Godel {
    fn new() -> Self {
        Self {
            godel_map: HashMap::new(),
            next_godel: 1,
        }
    }

    fn compute_path_godel(&self, path: &str) -> u64 {
        // Compute Gödel number using prime factorization
        let primes: [u64; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
        let mut godel_number = 1u64;

        for (i, c) in path.chars().enumerate() {
            if i < primes.len() {
                let char_code = (c as u32) as u64;
                let prime_power = primes[i].pow((char_code % 10) as u32); // Limit exponent
                godel_number = godel_number.saturating_mul(prime_power);

                // Use modular arithmetic to prevent overflow
                if godel_number > 1_000_000_000 {
                    godel_number %= 1_000_000_007;
                }
            }
        }

        godel_number
    }

    fn print_analysis(&self) {
        let path = "src/helloworld.rs";
        println!("🔄 Gödel Number Analysis:");
        println!("\n📝 Computing Gödel number for: {}", path);

        let godel_num = self.compute_path_godel(path);
        println!("\n✨ Gödel number of '{}': {}", path, godel_num);

        println!("\n📊 Character breakdown:");
        let primes: [u64; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];

        for (i, c) in path.chars().enumerate() {
            let char_code = c as u32;
            let prime = if i < primes.len() { primes[i] } else { 2 };
            let exponent = char_code % 10;

            println!(
                "    [{}] '{}' (ASCII {}) → {}^{} = {}",
                i,
                c,
                char_code,
                prime,
                exponent,
                prime.pow(exponent)
            );
        }

        // Show the mathematical construction
        println!("\n🧮 Gödel construction:");
        println!("    G('{}') = ∏ p_i^(ASCII(c_i) mod 10)", path);
        println!("    Where p_i are consecutive primes");

        // Compare with other paths
        let other_paths = ["src/main.rs", "Cargo.toml", "README.md"];
        println!("\n📈 Comparison with other paths:");

        for other_path in other_paths {
            let other_godel = self.compute_path_godel(other_path);
            println!("    '{}' → Gödel #{}", other_path, other_godel);
        }
    }
}

fn main() {
    let analyzer = Kleene2Markov2Godel::new();

    println!("🚀 Computing Gödel Number for src/helloworld.rs");

    analyzer.print_analysis();
}
