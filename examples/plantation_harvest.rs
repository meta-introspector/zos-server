use zos_server::plantation_filter::PlantationFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌾 Starting Plantation Harvest...");

    let filter = PlantationFilter::new("/home/mdupont/nix/index/allrs.txt");
    filter.generate_security_lattice()?;

    println!("✅ Security lattice generated from 1.4M Rust files!");
    Ok(())
}
