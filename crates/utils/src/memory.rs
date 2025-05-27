use std::sync::atomic::{AtomicUsize, Ordering};

/// Memory usage tracking for debugging and optimization
pub struct MemoryTracker {
    total_allocated: AtomicUsize,
    peak_allocated: AtomicUsize,
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTracker {
    pub const fn new() -> Self {
        Self {
            total_allocated: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
        }
    }
    
    pub fn allocated(&self, size: usize) {
        let current = self.total_allocated.fetch_add(size, Ordering::Relaxed) + size;
        
        // Update peak if necessary
        let mut peak = self.peak_allocated.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_allocated.compare_exchange_weak(
                peak, 
                current, 
                Ordering::Relaxed, 
                Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }
    }
    
    pub fn deallocated(&self, size: usize) {
        self.total_allocated.fetch_sub(size, Ordering::Relaxed);
    }
    
    pub fn current_usage(&self) -> usize {
        self.total_allocated.load(Ordering::Relaxed)
    }
    
    pub fn peak_usage(&self) -> usize {
        self.peak_allocated.load(Ordering::Relaxed)
    }
    
    pub fn reset_peak(&self) {
        let current = self.current_usage();
        self.peak_allocated.store(current, Ordering::Relaxed);
    }
}

/// Global memory tracker instance
pub static MEMORY_TRACKER: MemoryTracker = MemoryTracker::new();

/// Get current memory usage statistics
pub fn get_memory_stats() -> MemoryStats {
    MemoryStats {
        current_bytes: MEMORY_TRACKER.current_usage(),
        peak_bytes: MEMORY_TRACKER.peak_usage(),
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_bytes: usize,
    pub peak_bytes: usize,
}

impl MemoryStats {
    pub fn current_mb(&self) -> f64 {
        self.current_bytes as f64 / (1024.0 * 1024.0)
    }
    
    pub fn peak_mb(&self) -> f64 {
        self.peak_bytes as f64 / (1024.0 * 1024.0)
    }
}
