// 🧟 ENHANCED SHIM WITH P2P FEEDERS: Distributed compilation cluster
use std::env;
use std::process::Command;
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;
use libloading::Library;
use std::net::TcpStream;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Log build order as before
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("build_order.log")?;
    writeln!(log, "{}", args.join(" "))?;
    
    // Extract source file and target from rustc args
    let source_file = extract_source_file(&args);
    let target_arch = extract_target_arch(&args);
    
    if let Some(source) = source_file {
        // Run analysis feeders
        let analysis_data = run_analysis_feeders(&source)?;
        
        // Send to P2P cluster via libp2p2
        send_to_p2p_cluster(&source, &target_arch, &analysis_data)?;
        
        // Check if we should cross-compile
        if target_arch.contains("arm64") || target_arch.contains("aarch64") {
            cross_compile_arm64(&args)?;
        } else {
            // Regular compilation with zombie driver
            call_zombie_driver(&args)?;
        }
    } else {
        call_zombie_driver(&args)?;
    }
    
    Ok(())
}

fn extract_source_file(args: &[String]) -> Option<String> {
    args.iter()
        .find(|arg| arg.ends_with(".rs") && !arg.starts_with("-"))
        .cloned()
}

fn extract_target_arch(args: &[String]) -> String {
    args.iter()
        .position(|arg| arg == "--target")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "x86_64-unknown-linux-gnu".to_string())
}

fn run_analysis_feeders(source_file: &str) -> Result<AnalysisData, Box<dyn std::error::Error>> {
    let source = read_to_string(source_file)?;
    
    // Char analysis feeder
    let char_plugin = unsafe { Library::new("lib-zombie/target/debug/libchar_analyzer.so")? };
    let char_analyze = unsafe { 
        char_plugin.get::<unsafe extern "C" fn(*const i8) -> *const i8>(b"analyze_chars")? 
    };
    
    // Syn analysis feeder
    let syn_plugin = unsafe { Library::new("syn-analyzer/target/debug/libsyn_analyzer.so")? };
    let syn_analyze = unsafe { 
        syn_plugin.get::<unsafe extern "C" fn(*const i8) -> *const i8>(b"parse_and_analyze")? 
    };
    
    let c_source = std::ffi::CString::new(source)?;
    
    let char_result = unsafe { 
        let ptr = char_analyze(c_source.as_ptr());
        std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string()
    };
    
    let syn_result = unsafe {
        let ptr = syn_analyze(c_source.as_ptr());
        std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string()
    };
    
    Ok(AnalysisData {
        file: source_file.to_string(),
        char_analysis: char_result,
        syn_analysis: syn_result,
        timestamp: std::time::SystemTime::now(),
    })
}

fn send_to_p2p_cluster(source_file: &str, target_arch: &str, analysis: &AnalysisData) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to P2P cluster coordinator
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    
    let message = json!({
        "type": "compilation_task",
        "source_file": source_file,
        "target_arch": target_arch,
        "analysis": {
            "char_data": analysis.char_analysis,
            "syn_data": analysis.syn_analysis,
            "timestamp": analysis.timestamp.duration_since(std::time::UNIX_EPOCH)?.as_secs()
        },
        "node_id": get_node_id(),
        "capabilities": ["x86_64", "arm64", "analysis"]
    });
    
    use std::io::Write;
    stream.write_all(message.to_string().as_bytes())?;
    
    println!("📡 Sent compilation task to P2P cluster");
    Ok(())
}

fn cross_compile_arm64(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Cross-compiling for ARM64...");
    
    // Load ARM64 cross-compiler driver
    let arm_driver = unsafe { 
        Library::new("rustc_driver_arm64.so")
            .or_else(|_| Library::new("rustc_driver_self.so"))? 
    };
    
    let rustc_main = unsafe { 
        arm_driver.get::<unsafe extern "C" fn(i32, *const *const i8) -> i32>(b"rustc_driver_main")? 
    };
    
    // Convert args to C
    let c_args: Vec<std::ffi::CString> = args.iter()
        .map(|s| std::ffi::CString::new(s.as_str()).unwrap())
        .collect();
    let c_ptrs: Vec<*const i8> = c_args.iter().map(|s| s.as_ptr()).collect();
    
    let result = unsafe { rustc_main(c_ptrs.len() as i32, c_ptrs.as_ptr()) };
    
    if result == 0 {
        println!("✅ ARM64 cross-compilation successful");
    } else {
        println!("❌ ARM64 cross-compilation failed: {}", result);
    }
    
    std::process::exit(result);
}

fn call_zombie_driver(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("./zombie_callback")
        .args(&args[1..])
        .status()?;
    
    std::process::exit(status.code().unwrap_or(1));
}

fn get_node_id() -> String {
    format!("zombie_node_{}", std::process::id())
}

#[derive(Debug)]
struct AnalysisData {
    file: String,
    char_analysis: String,
    syn_analysis: String,
    timestamp: std::time::SystemTime,
}

// P2P Cluster Coordinator
fn start_p2p_cluster_coordinator() -> Result<(), Box<dyn std::error::Error>> {
    use std::net::{TcpListener, TcpStream};
    use std::io::Read;
    use std::thread;
    
    println!("🌐 Starting P2P compilation cluster coordinator...");
    
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let mut cluster_nodes = Vec::new();
    
    for stream in listener.incoming() {
        let stream = stream?;
        let nodes = cluster_nodes.clone();
        
        thread::spawn(move || {
            handle_cluster_node(stream, nodes);
        });
    }
    
    Ok(())
}

fn handle_cluster_node(mut stream: TcpStream, mut cluster_nodes: Vec<String>) {
    let mut buffer = [0; 4096];
    if let Ok(size) = stream.read(&mut buffer) {
        let message = String::from_utf8_lossy(&buffer[..size]);
        
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&message) {
            match data["type"].as_str() {
                Some("compilation_task") => {
                    println!("📨 Received compilation task from {}", data["node_id"]);
                    
                    // Distribute to available nodes based on target_arch
                    let target = data["target_arch"].as_str().unwrap_or("x86_64");
                    distribute_compilation_task(&data, target, &cluster_nodes);
                }
                Some("node_register") => {
                    let node_id = data["node_id"].as_str().unwrap_or("unknown");
                    cluster_nodes.push(node_id.to_string());
                    println!("🤝 Registered new cluster node: {}", node_id);
                }
                _ => {}
            }
        }
    }
}

fn distribute_compilation_task(task: &serde_json::Value, target_arch: &str, nodes: &[String]) {
    println!("🔄 Distributing {} compilation task to {} nodes", target_arch, nodes.len());
    
    // Find nodes capable of target architecture
    let capable_nodes: Vec<_> = nodes.iter()
        .filter(|node| node.contains("arm64") || target_arch.contains("x86_64"))
        .collect();
    
    if !capable_nodes.is_empty() {
        println!("✅ Found {} capable nodes for {}", capable_nodes.len(), target_arch);
    } else {
        println!("⚠️ No capable nodes found for {}", target_arch);
    }
}
