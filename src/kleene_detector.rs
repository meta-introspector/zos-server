use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KleenePattern {
    pub name: String,
    pub regex: String,
    pub closure_depth: usize,
    pub eigenvalue: f64,
}

pub struct KleeneAlgebraDetector {
    pub patterns: Vec<KleenePattern>,
}

impl KleeneAlgebraDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // Code readers (L* - Language closure)
                KleenePattern {
                    name: "file_reader_star".to_string(),
                    regex: r"(File::open|fs::read|read_to_string)\s*\(.*\)".to_string(),
                    closure_depth: 1,
                    eigenvalue: 1.0,
                },
                // Parsers (P* - Parser closure)
                KleenePattern {
                    name: "parser_star".to_string(),
                    regex: r"(syn::parse|parse_file|TokenStream|parse_quote)\s*[!<(]".to_string(),
                    closure_depth: 2,
                    eigenvalue: 2.0,
                },
                // Code generators (G* - Generator closure)
                KleenePattern {
                    name: "generator_star".to_string(),
                    regex: r"(quote!|proc_macro|TokenStream::from|to_tokens)".to_string(),
                    closure_depth: 3,
                    eigenvalue: 4.0,
                },
                // AST manipulators (A* - AST closure)
                KleenePattern {
                    name: "ast_star".to_string(),
                    regex: r"(visit_mut|fold|transform|ast::)".to_string(),
                    closure_depth: 4,
                    eigenvalue: 8.0,
                },
                // Meta-programmers (M* - Meta closure) - THE EIGENVECTOR!
                KleenePattern {
                    name: "meta_star".to_string(),
                    regex: r"(macro_rules!|#\[proc_macro|derive\(|#\[derive)".to_string(),
                    closure_depth: 5,
                    eigenvalue: 16.0,
                },
            ],
        }
    }

    pub fn detect_kleene_eigenvector(&self, content: &str) -> Vec<f64> {
        let mut eigenvalues = Vec::new();

        for pattern in &self.patterns {
            let matches = content.matches(&pattern.regex).count();
            let closure_score = matches as f64 * pattern.eigenvalue;
            eigenvalues.push(closure_score);
        }

        // Normalize to unit eigenvector
        let magnitude: f64 = eigenvalues.iter().map(|x| x * x).sum::<f64>().sqrt();
        if magnitude > 0.0 {
            eigenvalues.iter().map(|x| x / magnitude).collect()
        } else {
            eigenvalues
        }
    }

    pub fn find_dominant_eigenvector(&self, all_vectors: &[Vec<f64>]) -> Vec<f64> {
        if all_vectors.is_empty() {
            return vec![0.0; self.patterns.len()];
        }

        // Power iteration to find dominant eigenvector
        let mut dominant = vec![1.0; self.patterns.len()];

        for _ in 0..10 {
            // 10 iterations
            let mut next = vec![0.0; self.patterns.len()];

            for vector in all_vectors {
                for (i, &val) in vector.iter().enumerate() {
                    next[i] += val * dominant[i];
                }
            }

            // Normalize
            let magnitude: f64 = next.iter().map(|x| x * x).sum::<f64>().sqrt();
            if magnitude > 0.0 {
                dominant = next.iter().map(|x| x / magnitude).collect();
            }
        }

        dominant
    }

    pub fn classify_convergence_type(&self, eigenvector: &[f64]) -> String {
        let max_idx = eigenvector
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        match max_idx {
            0 => "File Reader Convergence".to_string(),
            1 => "Parser Convergence".to_string(),
            2 => "Code Generator Convergence".to_string(),
            3 => "AST Manipulator Convergence".to_string(),
            4 => "Meta-Programming Convergence (EIGENVECTOR!)".to_string(),
            _ => "Unknown Convergence".to_string(),
        }
    }
}
