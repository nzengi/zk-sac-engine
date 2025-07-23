use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tracing::{info, debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub block_production_time_ms: u64,
    pub proof_generation_time_ms: u64,
    pub validation_time_ms: u64,
    pub transactions_per_second: f64,
    pub proof_size_bytes: usize,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub network_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBenchmark {
    pub timestamp: u64,
    pub block_number: u64,
    pub transaction_count: u64,
    pub metrics: PerformanceMetrics,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct PerformanceMonitor {
    start_time: Instant,
    benchmarks: Vec<SystemBenchmark>,
    active_timers: HashMap<String, Instant>,
    error_counts: HashMap<String, u32>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        info!("📊 Initializing Performance Monitor");
        Self {
            start_time: Instant::now(),
            benchmarks: Vec::new(),
            active_timers: HashMap::new(),
            error_counts: HashMap::new(),
        }
    }

    pub fn start_timer(&mut self, operation: &str) {
        self.active_timers.insert(operation.to_string(), Instant::now());
        debug!("⏱️  Started timer for: {}", operation);
    }

    pub fn end_timer(&mut self, operation: &str) -> Duration {
        if let Some(start_time) = self.active_timers.remove(operation) {
            let duration = start_time.elapsed();
            debug!("⏱️  {} completed in {:?}", operation, duration);
            duration
        } else {
            warn!("⚠️  No active timer found for: {}", operation);
            Duration::from_millis(0)
        }
    }

    pub fn record_error(&mut self, error_type: &str) {
        *self.error_counts.entry(error_type.to_string()).or_insert(0) += 1;
        warn!("❌ Recorded error: {} (total: {})", error_type, self.error_counts[error_type]);
    }

    pub fn create_benchmark(
        &mut self,
        block_number: u64,
        transaction_count: u64,
        block_production_time: Duration,
        proof_generation_time: Duration,
        validation_time: Duration,
        proof_size: usize,
    ) -> SystemBenchmark {
        // Calculate TPS
        let total_time_seconds = block_production_time.as_secs_f64() + 
                                proof_generation_time.as_secs_f64() + 
                                validation_time.as_secs_f64();
        let tps = if total_time_seconds > 0.0 {
            transaction_count as f64 / total_time_seconds
        } else {
            0.0
        };

        // Get system metrics
        let (memory_mb, cpu_percent) = self.get_system_metrics();

        let metrics = PerformanceMetrics {
            block_production_time_ms: block_production_time.as_millis() as u64,
            proof_generation_time_ms: proof_generation_time.as_millis() as u64,
            validation_time_ms: validation_time.as_millis() as u64,
            transactions_per_second: tps,
            proof_size_bytes: proof_size,
            memory_usage_mb: memory_mb,
            cpu_usage_percent: cpu_percent,
            network_latency_ms: 0, // TODO: Implement network monitoring
        };

        let errors: Vec<String> = self.error_counts.iter()
            .map(|(error_type, count)| format!("{}: {}", error_type, count))
            .collect();

        let benchmark = SystemBenchmark {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            block_number,
            transaction_count,
            metrics,
            errors,
        };

        self.benchmarks.push(benchmark.clone());
        
        info!("📊 Benchmark recorded for block {}", block_number);
        info!("   ⚡ Block production: {:?}", block_production_time);
        info!("   🔧 Proof generation: {:?}", proof_generation_time);
        info!("   🔍 Validation: {:?}", validation_time);
        info!("   🚀 TPS: {:.2}", tps);
        info!("   📏 Proof size: {} bytes", proof_size);
        info!("   💾 Memory: {:.1} MB", memory_mb);
        info!("   🖥️  CPU: {:.1}%", cpu_percent);

        benchmark
    }

    pub fn get_performance_summary(&self) -> PerformanceSummary {
        if self.benchmarks.is_empty() {
            return PerformanceSummary::default();
        }

        let total_blocks = self.benchmarks.len() as u64;
        let total_transactions: u64 = self.benchmarks.iter()
            .map(|b| b.transaction_count)
            .sum();

        let avg_block_time: f64 = self.benchmarks.iter()
            .map(|b| b.metrics.block_production_time_ms as f64)
            .sum::<f64>() / total_blocks as f64;

        let avg_proof_time: f64 = self.benchmarks.iter()
            .map(|b| b.metrics.proof_generation_time_ms as f64)
            .sum::<f64>() / total_blocks as f64;

        let avg_tps: f64 = self.benchmarks.iter()
            .map(|b| b.metrics.transactions_per_second)
            .sum::<f64>() / total_blocks as f64;

        let max_tps = self.benchmarks.iter()
            .map(|b| b.metrics.transactions_per_second)
            .fold(0.0f64, f64::max);

        let avg_proof_size: f64 = self.benchmarks.iter()
            .map(|b| b.metrics.proof_size_bytes as f64)
            .sum::<f64>() / total_blocks as f64;

        let total_runtime = self.start_time.elapsed();

        PerformanceSummary {
            total_blocks,
            total_transactions,
            total_runtime_seconds: total_runtime.as_secs(),
            average_block_time_ms: avg_block_time,
            average_proof_time_ms: avg_proof_time,
            average_tps: avg_tps,
            max_tps,
            average_proof_size_bytes: avg_proof_size as usize,
            total_errors: self.error_counts.values().sum(),
        }
    }

    pub fn print_performance_report(&self) {
        let summary = self.get_performance_summary();
        
        info!("📊 PERFORMANCE REPORT");
        info!("==========================================");
        info!("🔗 Total blocks processed: {}", summary.total_blocks);
        info!("📝 Total transactions: {}", summary.total_transactions);
        info!("⏰ Total runtime: {} seconds", summary.total_runtime_seconds);
        info!("⚡ Average block time: {:.2} ms", summary.average_block_time_ms);
        info!("🔧 Average proof time: {:.2} ms", summary.average_proof_time_ms);
        info!("🚀 Average TPS: {:.2}", summary.average_tps);
        info!("🏆 Peak TPS: {:.2}", summary.max_tps);
        info!("📏 Average proof size: {} bytes", summary.average_proof_size_bytes);
        info!("❌ Total errors: {}", summary.total_errors);
        info!("==========================================");

        if !self.error_counts.is_empty() {
            info!("🔍 Error breakdown:");
            for (error_type, count) in &self.error_counts {
                info!("   {} errors: {}", error_type, count);
            }
        }
    }

    pub fn export_benchmarks(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.benchmarks)
    }

    pub fn get_latest_benchmark(&self) -> Option<&SystemBenchmark> {
        self.benchmarks.last()
    }

    pub fn get_benchmarks_since(&self, since_block: u64) -> Vec<&SystemBenchmark> {
        self.benchmarks.iter()
            .filter(|b| b.block_number >= since_block)
            .collect()
    }

    // Helper method to get system metrics
    fn get_system_metrics(&self) -> (f64, f64) {
        // In a real implementation, this would use system APIs to get actual metrics
        // For now, return simulated values
        let memory_mb = 128.0 + (rand::random::<f64>() * 64.0); // 128-192 MB
        let cpu_percent = 15.0 + (rand::random::<f64>() * 25.0); // 15-40% CPU
        (memory_mb, cpu_percent)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub total_runtime_seconds: u64,
    pub average_block_time_ms: f64,
    pub average_proof_time_ms: f64,
    pub average_tps: f64,
    pub max_tps: f64,
    pub average_proof_size_bytes: usize,
    pub total_errors: u32,
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            total_blocks: 0,
            total_transactions: 0,
            total_runtime_seconds: 0,
            average_block_time_ms: 0.0,
            average_proof_time_ms: 0.0,
            average_tps: 0.0,
            max_tps: 0.0,
            average_proof_size_bytes: 0,
            total_errors: 0,
        }
    }
}

// Performance testing utilities
pub struct PerformanceTest {
    monitor: PerformanceMonitor,
}

impl PerformanceTest {
    pub fn new() -> Self {
        Self {
            monitor: PerformanceMonitor::new(),
        }
    }

    pub async fn run_stress_test(
        &mut self,
        blocks_to_produce: u64,
        transactions_per_block: u64,
    ) -> PerformanceSummary {
        info!("🚀 Starting stress test: {} blocks, {} tx/block", blocks_to_produce, transactions_per_block);
        
        for block_num in 1..=blocks_to_produce {
            self.monitor.start_timer("block_production");
            
            // Simulate block production
            tokio::time::sleep(Duration::from_millis(10 + rand::random::<u64>() % 20)).await;
            let block_time = self.monitor.end_timer("block_production");
            
            self.monitor.start_timer("proof_generation");
            
            // Simulate proof generation (longer for more transactions)
            let proof_delay = 50 + (transactions_per_block * 2);
            tokio::time::sleep(Duration::from_millis(proof_delay)).await;
            let proof_time = self.monitor.end_timer("proof_generation");
            
            self.monitor.start_timer("validation");
            
            // Simulate validation
            tokio::time::sleep(Duration::from_millis(5 + rand::random::<u64>() % 10)).await;
            let validation_time = self.monitor.end_timer("validation");
            
            // Simulate occasional errors
            if rand::random::<f64>() < 0.05 { // 5% error rate
                self.monitor.record_error("network_timeout");
            }
            
            // Create benchmark
            let proof_size = 1024 + (transactions_per_block * 32) as usize;
            self.monitor.create_benchmark(
                block_num,
                transactions_per_block,
                block_time,
                proof_time,
                validation_time,
                proof_size,
            );
            
            if block_num % 10 == 0 {
                info!("📊 Completed {} blocks", block_num);
            }
        }
        
        let summary = self.monitor.get_performance_summary();
        self.monitor.print_performance_report();
        
        summary
    }

    pub fn get_monitor(&self) -> &PerformanceMonitor {
        &self.monitor
    }

    pub fn get_monitor_mut(&mut self) -> &mut PerformanceMonitor {
        &mut self.monitor
    }
} 