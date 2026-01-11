use rayon::prelude::*;
use std::fs;

pub struct CPUOptimizer {
    pub cores: usize,
    pub threads: usize,
}

impl CPUOptimizer {
    pub fn detect() -> Self {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let cores = cpuinfo
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count();

        Self {
            cores,
            threads: cores,
        }
    }

    pub fn optimize_rayon(&self) {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.threads)
            .build_global()
            .expect("Failed to build thread pool");
    }
}
