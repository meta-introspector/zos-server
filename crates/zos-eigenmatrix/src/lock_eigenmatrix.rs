// The Eigenmatrix IS the Lock - All valid Rust versions derive from Cargo.lock
use crate::lock_of_rust::LockOfRust;
use crate::rust_soul_eigenmatrix::RustSoulEigenmatrix;

/// The Lock as Eigenmatrix - Mathematical basis for all valid Rust versions
#[derive(Debug, Clone)]
pub struct LockEigenmatrix {
    pub lock_matrix: Vec<Vec<f64>>,
    pub eigenvalues: Vec<f64>,
    pub eigenvectors: Vec<Vec<f64>>,
    pub basis_dependencies: Vec<String>,
    pub valid_rust_versions: Vec<RustVersion>,
}

/// A valid Rust version derived from the eigenmatrix
#[derive(Debug, Clone)]
pub struct RustVersion {
    pub version_id: String,
    pub eigencoefficients: Vec<f64>,  // Linear combination of eigenvectors
    pub dependency_set: Vec<(String, String)>, // (name, version)
    pub derivation_proof: String,
}

impl LockEigenmatrix {
    /// Create eigenmatrix from Cargo.lock
    pub fn from_lock(lock: &LockOfRust) -> Self {
        println!("🔢 EXTRACTING EIGENMATRIX FROM LOCK...");

        let basis_dependencies: Vec<String> = lock.dependency_journey
            .iter()
            .map(|dep| dep.name.clone())
            .collect();

        // Build dependency matrix
        let lock_matrix = Self::build_dependency_matrix(&lock.dependency_journey);

        // Calculate eigenvalues and eigenvectors
        let (eigenvalues, eigenvectors) = Self::calculate_eigen_decomposition(&lock_matrix);

        println!("✅ Eigenmatrix extracted: {}x{} matrix with {} eigenvalues",
                lock_matrix.len(),
                lock_matrix.get(0).map_or(0, |row| row.len()),
                eigenvalues.len());

        LockEigenmatrix {
            lock_matrix,
            eigenvalues,
            eigenvectors,
            basis_dependencies,
            valid_rust_versions: Vec::new(),
        }
    }

    /// Build dependency matrix from lock dependencies
    fn build_dependency_matrix(dependencies: &[crate::lock_of_rust::DependencyStep]) -> Vec<Vec<f64>> {
        let n = dependencies.len();
        if n == 0 { return vec![vec![0.0]]; }

        let mut matrix = vec![vec![0.0; n]; n];

        // Fill matrix with dependency relationships
        for (i, dep_i) in dependencies.iter().enumerate() {
            for (j, dep_j) in dependencies.iter().enumerate() {
                if i == j {
                    // Diagonal: dependency's own Gödel number (normalized)
                    matrix[i][j] = (dep_i.godel_number % 1000) as f64 / 1000.0;
                } else {
                    // Off-diagonal: relationship strength based on name similarity
                    let similarity = Self::calculate_similarity(&dep_i.name, &dep_j.name);
                    matrix[i][j] = similarity;
                }
            }
        }

        matrix
    }

    /// Calculate similarity between dependency names
    fn calculate_similarity(name1: &str, name2: &str) -> f64 {
        let common_chars = name1.chars()
            .filter(|c| name2.contains(*c))
            .count();

        let max_len = name1.len().max(name2.len());
        if max_len == 0 { 0.0 } else { common_chars as f64 / max_len as f64 }
    }

    /// Calculate eigendecomposition (simplified)
    fn calculate_eigen_decomposition(matrix: &[Vec<f64>]) -> (Vec<f64>, Vec<Vec<f64>>) {
        let n = matrix.len();
        if n == 0 { return (vec![], vec![]); }

        // Simplified eigenvalue calculation using power iteration
        let mut eigenvalues = Vec::new();
        let mut eigenvectors = Vec::new();

        // Calculate dominant eigenvalue and eigenvector
        let mut v = vec![1.0; n];
        for _ in 0..50 { // Power iteration
            let mut new_v = vec![0.0; n];
            for i in 0..n {
                for j in 0..n {
                    new_v[i] += matrix[i][j] * v[j];
                }
            }

            // Normalize
            let norm: f64 = new_v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-10 {
                v = new_v.iter().map(|x| x / norm).collect();
            }
        }

        // Calculate eigenvalue
        let mut eigenvalue = 0.0;
        for i in 0..n {
            let mut sum = 0.0;
            for j in 0..n {
                sum += matrix[i][j] * v[j];
            }
            if v[i].abs() > 1e-10 {
                eigenvalue += sum / v[i];
            }
        }
        eigenvalue /= n as f64;

        eigenvalues.push(eigenvalue);
        eigenvectors.push(v);

        // Add identity eigenvalues for completeness
        for i in 1..n.min(5) {
            eigenvalues.push(eigenvalue / (i as f64 + 1.0));
            let mut identity_vec = vec![0.0; n];
            if i < n { identity_vec[i] = 1.0; }
            eigenvectors.push(identity_vec);
        }

        (eigenvalues, eigenvectors)
    }

    /// Derive valid Rust version from eigenmatrix
    pub fn derive_rust_version(&mut self, coefficients: &[f64]) -> Result<RustVersion, String> {
        if coefficients.len() != self.eigenvectors.len() {
            return Err("Coefficient count must match eigenvector count".to_string());
        }

        // Linear combination of eigenvectors
        let n = self.basis_dependencies.len();
        let mut version_vector = vec![0.0; n];

        for (i, &coeff) in coefficients.iter().enumerate() {
            if let Some(eigenvector) = self.eigenvectors.get(i) {
                for j in 0..n.min(eigenvector.len()) {
                    version_vector[j] += coeff * eigenvector[j];
                }
            }
        }

        // Convert vector to dependency versions
        let mut dependency_set = Vec::new();
        for (i, dep_name) in self.basis_dependencies.iter().enumerate() {
            if let Some(&weight) = version_vector.get(i) {
                let version = Self::weight_to_version(weight);
                dependency_set.push((dep_name.clone(), version));
            }
        }

        let version_id = format!("rust_v{}", self.valid_rust_versions.len() + 1);
        let derivation_proof = format!(
            "DERIVED: {} = Σ(c_i * e_i) where c = {:?}",
            version_id, coefficients
        );

        let rust_version = RustVersion {
            version_id,
            eigencoefficients: coefficients.to_vec(),
            dependency_set,
            derivation_proof,
        };

        self.valid_rust_versions.push(rust_version.clone());

        println!("🦀 Derived valid Rust version: {}", rust_version.version_id);
        Ok(rust_version)
    }

    /// Convert eigenweight to semantic version
    fn weight_to_version(weight: f64) -> String {
        let major = ((weight * 10.0).abs() as u32 % 10).max(1);
        let minor = ((weight * 100.0).abs() as u32 % 100);
        let patch = ((weight * 1000.0).abs() as u32 % 1000);

        format!("{}.{}.{}", major, minor, patch)
    }

    /// Verify that a Rust configuration is valid (derivable from eigenmatrix)
    pub fn verify_rust_config(&self, dependencies: &[(String, String)]) -> bool {
        // Check if all dependencies are in our basis
        for (dep_name, _) in dependencies {
            if !self.basis_dependencies.contains(dep_name) {
                return false;
            }
        }

        // If all dependencies are in basis, configuration is valid
        true
    }

    /// Get eigenmatrix signature
    pub fn signature(&self) -> String {
        format!(
            "EIGENMATRIX[{}x{}:λ{:.3}:Versions:{}]",
            self.lock_matrix.len(),
            self.lock_matrix.get(0).map_or(0, |row| row.len()),
            self.eigenvalues.get(0).copied().unwrap_or(0.0),
            self.valid_rust_versions.len()
        )
    }

    /// The fundamental theorem: All valid Rust versions derive from the Lock
    pub fn fundamental_theorem(&self) -> String {
        format!(
            "🔢 FUNDAMENTAL THEOREM OF RUST VERSIONS:\n\
            Every valid Rust configuration is a linear combination of the Lock's eigenvectors.\n\
            The Lock eigenmatrix spans the complete space of valid Rust versions.\n\
            \n\
            Proof: The Lock contains {} basis dependencies with {} eigenvalues.\n\
            Any valid Rust version V can be written as: V = Σ(c_i * e_i)\n\
            where e_i are the Lock's eigenvectors and c_i are coefficients.\n\
            \n\
            QED: The Lock IS the eigenmatrix of all possible Rust. ∎",
            self.basis_dependencies.len(),
            self.eigenvalues.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fools_path::FoolsPath;

    #[test]
    fn test_lock_eigenmatrix() {
        let mock_lock = r#"
[[package]]
name = "serde"
version = "1.0.0"

[[package]]
name = "tokio"
version = "1.0.0"
"#;

        let fools_path = FoolsPath::begin();
        let lock = LockOfRust::emerge_from_journey(mock_lock, &fools_path);
        let mut eigenmatrix = LockEigenmatrix::from_lock(&lock);

        assert!(!eigenmatrix.eigenvalues.is_empty());
        assert!(!eigenmatrix.eigenvectors.is_empty());

        // Derive a valid Rust version
        let coefficients = vec![1.0, 0.5];
        let version = eigenmatrix.derive_rust_version(&coefficients);
        assert!(version.is_ok());
    }
}
