// 性能优化和内存管理集成测试

use std::time::Duration;

#[test]
fn test_memory_management_integration() {
    // 测试帧缓冲区池的基本功能
    let pool = mira::memory::FrameBufferPool::new(1024, 2, 5);
    
    // 获取缓冲区
    let buffer1 = pool.get_buffer();
    assert_eq!(buffer1.len(), 1024);
    
    let stats = pool.get_stats();
    assert_eq!(stats.buffer_size, 1024);
    assert_eq!(stats.max_buffers, 5);
    
    // 归还缓冲区
    pool.return_buffer(buffer1);
    
    let stats_after = pool.get_stats();
    assert!(stats_after.available_count > 0);
}

#[test]
fn test_performance_monitoring_integration() {
    // 测试性能监控器的基本功能
    let mut monitor = mira::performance::PerformanceMonitor::new(
        10,
        Duration::from_secs(1),
        None,
    );
    
    // 记录一些性能数据
    let alert = monitor.record_frame(
        Duration::from_millis(16), // 60 FPS
        Duration::from_millis(8),  // 渲染时间
    );
    
    // 第一次记录不应该有警报
    assert!(alert.is_none());
    
    let stats = monitor.get_stats();
    assert!(stats.sample_count > 0);
}

#[test]
fn test_memory_monitor_integration() {
    // 测试内存监控器的基本功能
    let mut monitor = mira::memory::MemoryMonitor::new(
        Duration::from_millis(100),
        5,
        10.0,
    );
    
    // 更新内存监控
    let alert = monitor.update(100.0, 5);
    assert!(alert.is_none()); // 第一次更新不应该有警报
    
    let stats = monitor.get_stats();
    assert_eq!(stats.current_mb, 100.0);
}

#[test]
fn test_performance_alert_system() {
    use mira::performance::{PerformanceAlert, AlertSeverity};
    
    // 测试性能警报系统
    let alert = PerformanceAlert::LowFps {
        current: 25.0,
        threshold: 30.0,
    };
    
    assert_eq!(alert.severity(), AlertSeverity::Warning);
    assert!(alert.message().contains("FPS 过低"));
    
    // 测试严重警报
    let critical_alert = PerformanceAlert::LowFps {
        current: 10.0,
        threshold: 30.0,
    };
    
    assert_eq!(critical_alert.severity(), AlertSeverity::Critical);
}

#[test]
fn test_memory_leak_detection() {
    // 测试内存泄漏检测
    let mut monitor = mira::memory::MemoryMonitor::new(
        Duration::from_millis(1), // 很短的间隔用于测试
        10,
        5.0, // 低阈值用于测试
    );
    
    // 模拟内存持续增长
    std::thread::sleep(Duration::from_millis(2));
    monitor.update(100.0, 5);
    
    std::thread::sleep(Duration::from_millis(2));
    monitor.update(110.0, 5);
    
    std::thread::sleep(Duration::from_millis(2));
    let alert = monitor.update(120.0, 5);
    
    // 应该检测到内存泄漏
    match alert {
        Some(mira::memory::MemoryAlert::PossibleLeak { increase_mb, current_mb }) => {
            assert!(increase_mb > 0.0);
            assert_eq!(current_mb, 120.0);
        }
        _ => panic!("应该检测到内存泄漏"),
    }
}

#[test]
fn test_texture_manager_basic_functionality() {
    // 测试纹理管理器的基本功能
    let manager = mira::memory::TextureManager::new(
        5,
        Duration::from_secs(60),
    );
    
    let stats = manager.get_stats();
    assert_eq!(stats.cached_textures, 0);
    assert_eq!(stats.max_cached_textures, 5);
    assert_eq!(stats.total_usage, 0);
}

#[test]
fn test_performance_thresholds() {
    use mira::performance::PerformanceThresholds;
    
    // 测试默认性能阈值
    let thresholds = PerformanceThresholds::default();
    assert_eq!(thresholds.min_fps, 30.0);
    assert_eq!(thresholds.max_cpu_percent, 25.0);
    assert_eq!(thresholds.max_memory_mb, 200.0);
    assert_eq!(thresholds.max_frame_time_ms, 33.0);
    assert_eq!(thresholds.max_render_time_ms, 16.0);
}

#[test]
fn test_frame_buffer_pool_limits() {
    // 测试帧缓冲区池的限制
    let pool = mira::memory::FrameBufferPool::new(1024, 0, 2);
    
    // 获取超过最大限制的缓冲区
    let _buffer1 = pool.get_buffer();
    let _buffer2 = pool.get_buffer();
    let buffer3 = pool.get_buffer(); // 这个应该是临时缓冲区
    
    assert_eq!(buffer3.len(), 1024);
    
    let stats = pool.get_stats();
    assert_eq!(stats.allocated_count, 2); // 不应该超过最大限制
}

#[test]
fn test_performance_stats_calculation() {
    // 测试性能统计计算
    let mut monitor = mira::performance::PerformanceMonitor::new(
        10,
        Duration::from_secs(1),
        None,
    );
    
    // 添加一些性能数据
    monitor.record_frame(Duration::from_millis(16), Duration::from_millis(8));
    monitor.record_frame(Duration::from_millis(20), Duration::from_millis(10));
    monitor.record_frame(Duration::from_millis(18), Duration::from_millis(9));
    
    let stats = monitor.get_stats();
    assert!(stats.sample_count > 0);
    assert!(stats.avg_frame_time > 0.0);
    assert!(stats.avg_render_time > 0.0);
    assert!(stats.min_fps >= 0.0);
    assert!(stats.max_fps >= stats.min_fps);
}