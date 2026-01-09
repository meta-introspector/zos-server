// ZOS Server Enums - Zero Ontology System
// Consolidated enums from all P2P modules

#[derive(Debug, Clone, PartialEq)]
pub enum P2PVerb {
    // Core P2P operations
    Connect,
    Disconnect,
    SendMessage,
    ReceiveMessage,
    ListPeers,
    GetPeerInfo,
    
    // Dataset operations
    LoadDataset,
    UnloadDataset,
    QueryDataset,
    
    // Library operations
    LoadLibrary,
    UnloadLibrary,
    CallFunction,
    
    // Compilation operations
    CompileRust,
    LoadBinary,
    ExecuteFunction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LibVerb {
    Load,
    Unload,
    Call,
    List,
    Info,
}

#[derive(Debug, Clone)]
pub enum CompilationStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    PeerConnected(String),
    PeerDisconnected(String),
    MessageReceived(String, Vec<u8>),
    Error(String),
}
