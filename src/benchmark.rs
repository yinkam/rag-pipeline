use serde::Serialize;
use std::time::Instant;
use sysinfo::System;

#[derive(Debug)]
pub struct BenchmarkTracker {
    start_time: Instant,
    initial_memory: u64,
    cpu_count: usize,
    rayon_threads: usize,
    sys: System,
}

#[derive(Debug, Serialize)]
pub struct ResourceMetrics {
    pub memory_used_mb: String,
    pub total_memory_gb: String,
    pub cpu_count: usize,
    pub rayon_threads: usize,
}

impl BenchmarkTracker {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let initial_memory = sys.used_memory();
        let cpu_count = sys.cpus().len();
        let rayon_threads = (num_cpus::get() - 1).max(1);

        Self {
            start_time: Instant::now(),
            initial_memory,
            cpu_count,
            rayon_threads,
            sys,
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn elapsed_micros(&self) -> u128 {
        self.start_time.elapsed().as_micros()
    }

    pub fn elapsed_ms(&self) -> String {
        format!("{:.3}", self.start_time.elapsed().as_secs_f64() * 1000.0)
    }

    pub fn get_resources(&mut self) -> ResourceMetrics {
        self.sys.refresh_all();
        let final_memory = self.sys.used_memory();
        let memory_used_mb =
            (final_memory.saturating_sub(self.initial_memory)) as f64 / 1024.0 / 1024.0;
        let total_memory_gb = self.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

        ResourceMetrics {
            memory_used_mb: format!("{:.2}", memory_used_mb),
            total_memory_gb: format!("{:.2}", total_memory_gb),
            cpu_count: self.cpu_count,
            rayon_threads: self.rayon_threads,
        }
    }

    pub fn print_summary(&mut self, label: &str) {
        let resources = self.get_resources();
        let duration = self.elapsed();

        println!("⏱️  {}: {:.2?}", label, duration);
        println!("💾 Memory used: {} MB", resources.memory_used_mb);
        println!("🖥️  CPUs available: {}\n", resources.cpu_count);
    }
}

impl Default for BenchmarkTracker {
    fn default() -> Self {
        Self::new()
    }
}
