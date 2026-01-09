// ZOS Traits - Zero dependency trait foundation
// AGPL-3.0 License

/// Core plugin execution trait
pub trait Plugin {
    type Input;
    type Output;
    type Error;

    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

/// Security verification trait
pub trait SecurityVerifier {
    fn verify(&self) -> bool;
    fn security_level(&self) -> u8;
}

/// LMFDB orbit reference trait - we don't define orbits, LMFDB does
pub trait LMFDBOrbitRef {
    /// Reference to LMFDB orbit identifier
    fn lmfdb_orbit_id(&self) -> &str;

    /// LMFDB complexity class reference
    fn lmfdb_complexity_class(&self) -> &str;
}
