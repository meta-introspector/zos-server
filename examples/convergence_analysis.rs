use zos_server::convergence_analyzer::ConvergenceAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧬 Starting Convergence Analysis on 1.4M Rust files...");

    let mut analyzer = ConvergenceAnalyzer::new();
    analyzer.process_plantation("/home/mdupont/nix/index/allrs.txt")?;
    analyzer.report_convergence();

    println!("\n🎯 Kleene Algebra Eigenvector Discovery Complete!");
    Ok(())
}
