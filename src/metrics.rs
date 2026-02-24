use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

/// Server-wide metrics tracking
pub struct ServerMetrics {
    pub start_time: Instant,
    pub total_messages: AtomicU64,
    pub total_connections: AtomicUsize,
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_messages: AtomicU64::new(0),
            total_connections: AtomicUsize::new(0),
        }
    }

    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub fn increment_messages(&self) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_connections(&self) {
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_total_messages(&self) -> u64 {
        self.total_messages.load(Ordering::Relaxed)
    }

    pub fn get_total_connections(&self) -> u64 {
        self.total_connections.load(Ordering::Relaxed) as u64
    }

    /// Get memory usage in MB (approximation)
    pub fn memory_usage_mb(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
                let parts: Vec<&str> = statm.split_whitespace().collect();
                if let Some(rss_pages) = parts.get(1) {
                    if let Ok(pages) = rss_pages.parse::<usize>() {
                        // Each page is typically 4KB
                        return (pages * 4) / 1024; // Convert to MB
                    }
                }
            }
        }

        // Fallback: return 0 if we can't determine
        0
    }
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}
