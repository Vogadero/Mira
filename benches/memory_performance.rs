// 内存管理和性能优化基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mira::memory::FrameBufferPool;
use std::sync::Arc;
use std::time::Duration;

fn benchmark_frame_buffer_allocation(c: &mut Criterion) {
    c.bench_function("frame_buffer_direct_allocation", |b| {
        b.iter(|| {
            let buffer = vec![0u8; black_box(1920 * 1080 * 4)];
            black_box(buffer);
        });
    });
    
    let pool = Arc::new(FrameBufferPool::new(1920 * 1080 * 4, 5, 10));
    c.bench_function("frame_buffer_pool_allocation", |b| {
        let pool = pool.clone();
        b.iter(|| {
            let buffer = pool.get_buffer();
            black_box(&buffer);
            pool.return_buffer(buffer);
        });
    });
}

fn benchmark_performance_monitoring(c: &mut Criterion) {
    let mut monitor = mira::performance::PerformanceMonitor::new(
        1000,
        Duration::from_secs(10),
        None,
    );
    
    c.bench_function("performance_record_frame", |b| {
        b.iter(|| {
            let alert = monitor.record_frame(
                Duration::from_millis(black_box(16)),
                Duration::from_millis(black_box(8)),
            );
            black_box(alert);
        });
    });
}

fn benchmark_memory_monitoring(c: &mut Criterion) {
    let mut monitor = mira::memory::MemoryMonitor::new(
        Duration::from_millis(100),
        100,
        50.0,
    );
    
    c.bench_function("memory_monitor_update", |b| {
        b.iter(|| {
            let alert = monitor.update(black_box(150.0), black_box(10));
            black_box(alert);
        });
    });
}

fn benchmark_texture_manager(c: &mut Criterion) {
    let manager = mira::memory::TextureManager::new(10, Duration::from_secs(60));
    
    c.bench_function("texture_manager_stats", |b| {
        b.iter(|| {
            let stats = manager.get_stats();
            black_box(stats);
        });
    });
}

criterion_group!(
    benches,
    benchmark_frame_buffer_allocation,
    benchmark_performance_monitoring,
    benchmark_memory_monitoring,
    benchmark_texture_manager
);
criterion_main!(benches);