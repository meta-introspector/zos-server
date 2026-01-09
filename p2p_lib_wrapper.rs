use goblin::elf::Elf;
use libloading::{Library, Symbol};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LibVerb {
    LoadLib(String),
    ReloadLib,
    ListSymbols,
    ReadSymbol(String),
    InvokeSymbol(String),
    GetLibInfo,
    SaveResults(String),
}

#[derive(Debug)]
pub struct P2PLibWrapper {
    library: Option<Library>,
    lib_path: Option<String>,
    peer_id: PeerId,
    results: HashMap<String, Vec<u8>>,
}

impl P2PLibWrapper {
    pub fn new() -> Self {
        Self {
            library: None,
            lib_path: None,
            peer_id: PeerId::random(),
            results: HashMap::new(),
        }
    }

    pub fn execute_verb(&mut self, verb: LibVerb) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match verb {
            LibVerb::LoadLib(path) => self.load_lib(&path),
            LibVerb::ReloadLib => self.reload_lib(),
            LibVerb::ListSymbols => self.list_symbols(),
            LibVerb::ReadSymbol(name) => self.read_symbol(&name),
            LibVerb::InvokeSymbol(name) => self.invoke_symbol(&name),
            LibVerb::GetLibInfo => self.get_lib_info(),
            LibVerb::SaveResults(path) => self.save_results(&path),
        }
    }

    fn load_lib(&mut self, path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("🔌 Loading library: {}", path);

        if !Path::new(path).exists() {
            return Err(format!("Library not found: {}", path).into());
        }

        unsafe {
            let library = Library::new(path)?;
            let file_size = std::fs::metadata(path)?.len();

            self.library = Some(library);
            self.lib_path = Some(path.to_string());

            let result = format!("Loaded {} ({} bytes)", path, file_size);
            self.results.insert("load_result".to_string(), result.as_bytes().to_vec());

            println!("✅ Library loaded successfully");
            Ok(result.as_bytes().to_vec())
        }
    }

    fn reload_lib(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Some(path) = &self.lib_path.clone() {
            println!("🔄 Reloading library: {}", path);
            self.library = None;
            self.load_lib(path)
        } else {
            Err("No library loaded to reload".into())
        }
    }

    fn list_symbols(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("📋 Reading ELF symbols with Rust...");

        if self.library.is_none() {
            return Err("No library loaded".into());
        }

        let lib_path = self.lib_path.as_ref().unwrap();
        let buffer = std::fs::read(lib_path)?;

        match goblin::elf::Elf::parse(&buffer) {
            Ok(elf) => {
                let mut symbols = Vec::new();

                for sym in &elf.syms {
                    if let Some(name) = elf.strtab.get_at(sym.st_name) {
                        if !name.is_empty() && sym.st_value != 0 {
                            symbols.push(format!("{} @ 0x{:x}", name, sym.st_value));
                        }
                    }
                }

                let symbols_json = serde_json::to_vec(&symbols)?;
                self.results.insert("elf_symbols".to_string(), symbols_json.clone());

                println!("Found {} ELF symbols", symbols.len());
                Ok(symbols_json)
            }
            Err(e) => Err(format!("Failed to parse ELF: {}", e).into())
        }
    }

    fn read_symbol(&mut self, name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("🔍 Reading symbol: {}", name);

        let library = self.library.as_ref().ok_or("No library loaded")?;

        unsafe {
            if let Ok(symbol) = library.get::<Symbol<*const u8>>(name.as_bytes()) {
                let addr = symbol.into_raw().into_raw() as usize;
                let addr_bytes = addr.to_le_bytes().to_vec();

                self.results.insert(format!("symbol_{}", name), addr_bytes.clone());

                println!("✅ Symbol {} at address: 0x{:x}", name, addr);
                Ok(addr_bytes)
            } else {
                Err(format!("Symbol {} not found", name).into())
            }
        }
    }

    fn invoke_symbol(&mut self, name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("🚀 Invoking symbol: {}", name);

        let library = self.library.as_ref().ok_or("No library loaded")?;

        unsafe {
            if let Ok(symbol) = library.get::<Symbol<unsafe extern "C" fn() -> i32>>(name.as_bytes()) {
                let result = symbol();
                let result_bytes = result.to_le_bytes().to_vec();

                self.results.insert(format!("invoke_{}", name), result_bytes.clone());

                println!("✅ Symbol {} returned: {}", name, result);
                Ok(result_bytes)
            } else {
                Err(format!("Symbol {} not callable", name).into())
            }
        }
    }

    fn get_lib_info(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("📊 Getting library info...");

        let lib_path = self.lib_path.as_ref().ok_or("No library loaded")?;
        let metadata = std::fs::metadata(lib_path)?;

        let info = serde_json::json!({
            "path": lib_path,
            "size_bytes": metadata.len(),
            "size_gb": metadata.len() as f64 / 1_073_741_824.0,
            "peer_id": self.peer_id.to_string(),
            "results_count": self.results.len(),
        });

        let info_bytes = serde_json::to_vec(&info)?;
        self.results.insert("lib_info".to_string(), info_bytes.clone());

        Ok(info_bytes)
    }

    fn save_results(&self, path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("💾 Saving results to: {}", path);

        let results_json = serde_json::to_string_pretty(&self.results)?;
        std::fs::write(path, &results_json)?;

        let saved_msg = format!("Saved {} results to {}", self.results.len(), path);
        println!("✅ {}", saved_msg);

        Ok(saved_msg.as_bytes().to_vec())
    }
}

fn main() {
    println!("🚀 P2P Library Server Starting...");
    let mut server = P2PLibWrapper::new();

    // Load the rustc driver
    match server.load_lib("/mnt/data1/nix/vendor/rust/cargo2nix/submodules/rust/compiler/zombie_driver2/target/debug/deps/librustc_driver.so") {
        Ok(_) => println!("✅ Library loaded successfully"),
        Err(e) => println!("❌ Failed to load library: {}", e),
    }

    // Keep server running
    println!("🔄 Server running... Press Ctrl+C to stop");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
