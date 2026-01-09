// Packet Entropy Analyzer and User Complexity Scoring
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::entropy_scanner::EntropyScanner;

/// Packet entropy analyzer
pub struct PacketEntropyAnalyzer {
    entropy_scanner: EntropyScanner,
    user_scores: HashMap<String, UserComplexityScore>,
    packet_history: Vec<PacketAnalysis>,
    max_packet_entropy: f64,
}

#[derive(Debug, Clone)]
pub struct UserComplexityScore {
    pub user_id: String,
    pub total_requests: u64,
    pub avg_entropy: f64,
    pub max_entropy_seen: f64,
    pub complexity_score: f64,
    pub risk_level: RiskLevel,
    pub last_update: u64,
}

#[derive(Debug, Clone)]
pub struct PacketAnalysis {
    pub user_id: String,
    pub timestamp: u64,
    pub packet_size: usize,
    pub entropy: f64,
    pub complexity_indicators: Vec<String>,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,      // Normal user behavior
    Medium,   // Slightly suspicious
    High,     // Potentially malicious
    Critical, // Definitely malicious
}

impl PacketEntropyAnalyzer {
    pub fn new(max_packet_entropy: f64) -> Self {
        Self {
            entropy_scanner: EntropyScanner::new(max_packet_entropy),
            user_scores: HashMap::new(),
            packet_history: Vec::new(),
            max_packet_entropy,
        }
    }

    /// Analyze incoming packet and update user score
    pub fn analyze_packet(&mut self, user_id: &str, packet_data: &[u8]) -> PacketAnalysis {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let entropy_result = self.entropy_scanner.scan_entropy(packet_data);

        let mut complexity_indicators = Vec::new();
        let mut allowed = true;
        let mut reason = "Packet accepted".to_string();

        // Check entropy limit
        if entropy_result.overall_entropy > self.max_packet_entropy {
            allowed = false;
            reason = format!("Entropy {:.2} exceeds limit {:.2}",
                           entropy_result.overall_entropy, self.max_packet_entropy);
            complexity_indicators.push("high_entropy".to_string());
        }

        // Detect complexity patterns
        self.detect_complexity_patterns(packet_data, &mut complexity_indicators);

        // Check for suspicious patterns
        if self.contains_suspicious_patterns(packet_data) {
            complexity_indicators.push("suspicious_patterns".to_string());
            if allowed {
                reason = "Contains suspicious patterns".to_string();
            }
        }

        let analysis = PacketAnalysis {
            user_id: user_id.to_string(),
            timestamp,
            packet_size: packet_data.len(),
            entropy: entropy_result.overall_entropy,
            complexity_indicators,
            allowed,
            reason,
        };

        // Update user score
        self.update_user_score(user_id, &analysis);

        // Store analysis
        self.packet_history.push(analysis.clone());

        // Limit history size
        if self.packet_history.len() > 10000 {
            self.packet_history.drain(0..1000);
        }

        analysis
    }

    fn detect_complexity_patterns(&self, data: &[u8], indicators: &mut Vec<String>) {
        // Check for binary data
        if data.iter().any(|&b| b < 32 && b != 9 && b != 10 && b != 13) {
            indicators.push("binary_data".to_string());
        }

        // Check for base64 encoding
        if self.is_base64_like(data) {
            indicators.push("base64_encoded".to_string());
        }

        // Check for repeated patterns
        if self.has_repeated_patterns(data) {
            indicators.push("repeated_patterns".to_string());
        }

        // Check for compression signatures
        if self.has_compression_signatures(data) {
            indicators.push("compressed_data".to_string());
        }
    }

    fn contains_suspicious_patterns(&self, data: &[u8]) -> bool {
        let suspicious_strings = [
            b"eval(", b"exec(", b"system(", b"shell_exec(",
            b"<script", b"javascript:", b"data:text/html",
            b"../../../../", b"..\\..\\..\\",
        ];

        for pattern in &suspicious_strings {
            if data.windows(pattern.len()).any(|window| window == *pattern) {
                return true;
            }
        }

        false
    }

    fn is_base64_like(&self, data: &[u8]) -> bool {
        if data.len() < 4 || data.len() % 4 != 0 {
            return false;
        }

        data.iter().all(|&b| {
            (b >= b'A' && b <= b'Z') ||
            (b >= b'a' && b <= b'z') ||
            (b >= b'0' && b <= b'9') ||
            b == b'+' || b == b'/' || b == b'='
        })
    }

    fn has_repeated_patterns(&self, data: &[u8]) -> bool {
        if data.len() < 16 {
            return false;
        }

        // Check for 4-byte repeated patterns
        for i in 0..data.len().saturating_sub(12) {
            let pattern = &data[i..i+4];
            let mut count = 0;
            for j in (i+4..data.len().saturating_sub(3)).step_by(4) {
                if &data[j..j+4] == pattern {
                    count += 1;
                    if count >= 3 {
                        return true;
                    }
                } else {
                    break;
                }
            }
        }

        false
    }

    fn has_compression_signatures(&self, data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }

        // Check for common compression signatures
        let signatures = [
            &[0x1f, 0x8b], // gzip
            &[0x50, 0x4b], // zip
            &[0x42, 0x5a], // bzip2
            &[0xfd, 0x37], // xz
        ];

        for sig in &signatures {
            if data.starts_with(sig) {
                return true;
            }
        }

        false
    }

    fn update_user_score(&mut self, user_id: &str, analysis: &PacketAnalysis) {
        let score = self.user_scores.entry(user_id.to_string())
            .or_insert_with(|| UserComplexityScore {
                user_id: user_id.to_string(),
                total_requests: 0,
                avg_entropy: 0.0,
                max_entropy_seen: 0.0,
                complexity_score: 0.0,
                risk_level: RiskLevel::Low,
                last_update: analysis.timestamp,
            });

        // Update statistics
        score.total_requests += 1;
        score.max_entropy_seen = score.max_entropy_seen.max(analysis.entropy);
        score.last_update = analysis.timestamp;

        // Update average entropy (exponential moving average)
        let alpha = 0.1;
        score.avg_entropy = alpha * analysis.entropy + (1.0 - alpha) * score.avg_entropy;

        // Calculate complexity score
        score.complexity_score = self.calculate_complexity_score(score, analysis);

        // Update risk level
        score.risk_level = self.calculate_risk_level(score);
    }

    fn calculate_complexity_score(&self, score: &UserComplexityScore, analysis: &PacketAnalysis) -> f64 {
        let mut complexity = 0.0;

        // Entropy contribution
        complexity += analysis.entropy / 8.0 * 0.4;

        // Complexity indicators
        complexity += analysis.complexity_indicators.len() as f64 * 0.1;

        // Historical behavior
        complexity += (score.avg_entropy / 8.0) * 0.3;

        // Frequency penalty (too many requests)
        if score.total_requests > 1000 {
            complexity += 0.2;
        }

        complexity.min(1.0)
    }

    fn calculate_risk_level(&self, score: &UserComplexityScore) -> RiskLevel {
        if score.complexity_score >= 0.8 || score.max_entropy_seen > 7.0 {
            RiskLevel::Critical
        } else if score.complexity_score >= 0.6 || score.max_entropy_seen > 6.0 {
            RiskLevel::High
        } else if score.complexity_score >= 0.4 || score.max_entropy_seen > 5.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// Check if user should be blocked
    pub fn should_block_user(&self, user_id: &str) -> bool {
        if let Some(score) = self.user_scores.get(user_id) {
            matches!(score.risk_level, RiskLevel::Critical)
        } else {
            false
        }
    }

    /// Get user complexity report
    pub fn get_user_report(&self, user_id: &str) -> Option<String> {
        self.user_scores.get(user_id).map(|score| {
            format!(
                "USER COMPLEXITY REPORT\n\
                 User: {}\n\
                 Total Requests: {}\n\
                 Average Entropy: {:.2}\n\
                 Max Entropy: {:.2}\n\
                 Complexity Score: {:.2}\n\
                 Risk Level: {:?}\n\
                 Last Update: {}",
                score.user_id,
                score.total_requests,
                score.avg_entropy,
                score.max_entropy_seen,
                score.complexity_score,
                score.risk_level,
                score.last_update
            )
        })
    }

    /// Get system-wide statistics
    pub fn get_system_stats(&self) -> SystemStats {
        let total_users = self.user_scores.len();
        let total_packets = self.packet_history.len();
        let blocked_packets = self.packet_history.iter().filter(|p| !p.allowed).count();

        let risk_distribution = self.user_scores.values().fold(
            HashMap::new(),
            |mut acc, score| {
                *acc.entry(score.risk_level.clone()).or_insert(0) += 1;
                acc
            }
        );

        SystemStats {
            total_users,
            total_packets,
            blocked_packets,
            risk_distribution,
        }
    }
}

#[derive(Debug)]
pub struct SystemStats {
    pub total_users: usize,
    pub total_packets: usize,
    pub blocked_packets: usize,
    pub risk_distribution: HashMap<RiskLevel, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_entropy_packet() {
        let mut analyzer = PacketEntropyAnalyzer::new(4.0);
        let simple_packet = b"Hello, world!";

        let analysis = analyzer.analyze_packet("user1", simple_packet);
        assert!(analysis.allowed);
        assert!(analysis.entropy < 4.0);
    }

    #[test]
    fn test_high_entropy_packet() {
        let mut analyzer = PacketEntropyAnalyzer::new(4.0);
        let random_packet: Vec<u8> = (0..256).map(|i| i as u8).collect();

        let analysis = analyzer.analyze_packet("user1", &random_packet);
        assert!(!analysis.allowed);
        assert!(analysis.entropy > 4.0);
    }

    #[test]
    fn test_user_scoring() {
        let mut analyzer = PacketEntropyAnalyzer::new(4.0);

        // Send multiple packets
        for i in 0..10 {
            let packet = format!("Request {}", i).into_bytes();
            analyzer.analyze_packet("user1", &packet);
        }

        let score = analyzer.user_scores.get("user1").unwrap();
        assert_eq!(score.total_requests, 10);
        assert!(score.avg_entropy > 0.0);
    }
}
