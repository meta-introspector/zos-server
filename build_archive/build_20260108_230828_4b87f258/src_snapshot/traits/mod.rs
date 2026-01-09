// ZOS Server Traits - Zero Ontology System
// Common traits for P2P operations

use crate::enums::{P2PVerb, LibVerb, NetworkEvent};
use crate::structs::{PeerInfo, DatasetSeed, LoadedSo};
use std::collections::HashMap;

pub trait P2PNetwork {
    fn connect_peer(&mut self, peer_id: &str) -> Result<(), String>;
    fn disconnect_peer(&mut self, peer_id: &str) -> Result<(), String>;
    fn send_message(&mut self, peer_id: &str, message: &[u8]) -> Result<(), String>;
    fn list_peers(&self) -> Vec<PeerInfo>;
    fn handle_event(&mut self, event: NetworkEvent) -> Result<(), String>;
}

pub trait LibraryLoader {
    fn load_library(&mut self, path: &str) -> Result<String, String>;
    fn unload_library(&mut self, lib_id: &str) -> Result<(), String>;
    fn call_function(&mut self, lib_id: &str, func_name: &str, args: &[u8]) -> Result<Vec<u8>, String>;
    fn list_functions(&self, lib_id: &str) -> Result<Vec<String>, String>;
}

pub trait DatasetManager {
    fn load_dataset(&mut self, seed: DatasetSeed) -> Result<String, String>;
    fn unload_dataset(&mut self, dataset_id: &str) -> Result<(), String>;
    fn query_dataset(&self, dataset_id: &str, query: &str) -> Result<Vec<u8>, String>;
    fn list_datasets(&self) -> Vec<DatasetSeed>;
}

pub trait CompilationEngine {
    fn compile_rust(&mut self, source: &str) -> Result<Vec<u8>, String>;
    fn load_binary(&mut self, binary: &[u8]) -> Result<String, String>;
    fn execute_function(&mut self, binary_id: &str, func_name: &str, args: &[u8]) -> Result<Vec<u8>, String>;
}

pub trait P2PServer: P2PNetwork + LibraryLoader + DatasetManager {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn handle_verb(&mut self, verb: P2PVerb, args: &[u8]) -> Result<Vec<u8>, String>;
}
