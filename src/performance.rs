// 性能监控模块

use log::{debug, info, warn, error};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 性能监控器
pub struct PerformanceMonitor {
    /// FPS 计算器
    fps_calculator: FpsCalculator,
    /// CPU 使用率监控器
    cpu_monitor: CpuMonitor,
    /// 内存使用监控器
    memory_monitor: MemoryUsageMonitor,
    /// 性能历史记录
    performance_history: VecDeque<PerformanceSnapshot>,
    /// 最大历史记录数
    max_history: usize,
    /// 上次报告时间
    last_report: Instant,
    /// 报告间隔
    report_interval: Duration,
    /// 性能警告阈值
    thresholds: PerformanceThresholds,
}

/// 性能快照
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: Instant,
    pub fps: f32,
    pub cpu_percent: f32,
    pub memory_mb: f32,
    pub frame_time_ms: f32,
    pub render_time_ms: f32,
}

/// 性能阈值配置
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub min_fps: f32,
    pub max_cpu_percent: f32,
    pub max_memory_mb: f32,
    pub max_frame_time_ms: f32,
    pub max_render_time_ms: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            min_fps: 30.0,
            max_cpu_percent: 25.0,
            max_memory_mb: 200.0,
            max_frame_time_ms: 33.0, // 30 FPS = 33ms per frame
            max_render_time_ms: 16.0, // 留一半时间给其他操作
        }
    }
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(
        max_history: usize,
        report_interval: Duration,
        thresholds: Option<PerformanceThresholds>,
    ) -> Self {
        info!("创建性能监控器: 最大历史={}, 报告间隔={:?}", max_history, report_interval);
        
        Self {
            fps_calculator: FpsCalculator::new(Duration::from_secs(1)),
            cpu_monitor: CpuMonitor::new(),
            memory_monitor: MemoryUsageMonitor::new(),
            performance_history: VecDeque::with_capacity(max_history),
            max_history,
            last_report: Instant::now(),
            report_interval,
            thresholds: thresholds.unwrap_or_default(),
        }
    }
    
    /// 记录一帧的性能数据
    pub fn record_frame(&mut self, frame_time: Duration, render_time: Duration) -> Option<PerformanceAlert> {
        let now = Instant::now();
        
        // 更新 FPS 计算器
        let current_fps = self.fps_calculator.update();
        
        // 获取 CPU 和内存使用率
        let cpu_percent = self.cpu_monitor.get_usage();
        let memory_mb = self.memory_monitor.get_usage_mb();
        
        // 创建性能快照
        let snapshot = PerformanceSnapshot {
            timestamp: now,
            fps: current_fps,
            cpu_percent,
            memory_mb,
            frame_time_ms: frame_time.as_secs_f32() * 1000.0,
            render_time_ms: render_time.as_secs_f32() * 1000.0,
        };
        
        // 添加到历史记录
        self.performance_history.push_back(snapshot.clone());
        while self.performance_history.len() > self.max_history {
            self.performance_history.pop_front();
        }
        
        debug!("性能记录: FPS={:.1}, CPU={:.1}%, 内存={:.1}MB, 帧时间={:.1}ms, 渲染时间={:.1}ms",
               current_fps, cpu_percent, memory_mb, snapshot.frame_time_ms, snapshot.render_time_ms);
        
        // 检查性能警告
        let alert = self.check_performance_alerts(&snapshot);
        
        // 定期报告性能统计
        if now.duration_since(self.last_report) >= self.report_interval {
            self.report_performance_stats();
            self.last_report = now;
        }
        
        alert
    }
    
    /// 检查性能警告
    fn check_performance_alerts(&self, snapshot: &PerformanceSnapshot) -> Option<PerformanceAlert> {
        // 检查 FPS 过低
        if snapshot.fps < self.thresholds.min_fps {
            return Some(PerformanceAlert::LowFps {
                current: snapshot.fps,
                threshold: self.thresholds.min_fps,
            });
        }
        
        // 检查 CPU 使用率过高
        if snapshot.cpu_percent > self.thresholds.max_cpu_percent {
            return Some(PerformanceAlert::HighCpu {
                current: snapshot.cpu_percent,
                threshold: self.thresholds.max_cpu_percent,
            });
        }
        
        // 检查内存使用过高
        if snapshot.memory_mb > self.thresholds.max_memory_mb {
            return Some(PerformanceAlert::HighMemory {
                current: snapshot.memory_mb,
                threshold: self.thresholds.max_memory_mb,
            });
        }
        
        // 检查帧时间过长
        if snapshot.frame_time_ms > self.thresholds.max_frame_time_ms {
            return Some(PerformanceAlert::SlowFrame {
                current: snapshot.frame_time_ms,
                threshold: self.thresholds.max_frame_time_ms,
            });
        }
        
        // 检查渲染时间过长
        if snapshot.render_time_ms > self.thresholds.max_render_time_ms {
            return Some(PerformanceAlert::SlowRender {
                current: snapshot.render_time_ms,
                threshold: self.thresholds.max_render_time_ms,
            });
        }
        
        None
    }
    
    /// 报告性能统计信息
    fn report_performance_stats(&self) {
        if self.performance_history.is_empty() {
            return;
        }
        
        let stats = self.calculate_stats();
        
        info!("=== 性能统计报告 ===");
        info!("FPS: 当前={:.1}, 平均={:.1}, 最小={:.1}, 最大={:.1}", 
              stats.current_fps, stats.avg_fps, stats.min_fps, stats.max_fps);
        info!("CPU: 当前={:.1}%, 平均={:.1}%, 最大={:.1}%", 
              stats.current_cpu, stats.avg_cpu, stats.max_cpu);
        info!("内存: 当前={:.1}MB, 平均={:.1}MB, 最大={:.1}MB", 
              stats.current_memory, stats.avg_memory, stats.max_memory);
        info!("帧时间: 当前={:.1}ms, 平均={:.1}ms, 最大={:.1}ms", 
              stats.current_frame_time, stats.avg_frame_time, stats.max_frame_time);
        info!("渲染时间: 当前={:.1}ms, 平均={:.1}ms, 最大={:.1}ms", 
              stats.current_render_time, stats.avg_render_time, stats.max_render_time);
        info!("样本数量: {}", stats.sample_count);
        info!("==================");
        
        // 检查是否有性能问题
        if stats.avg_fps < self.thresholds.min_fps {
            warn!("平均 FPS ({:.1}) 低于阈值 ({:.1})", stats.avg_fps, self.thresholds.min_fps);
        }
        if stats.avg_cpu > self.thresholds.max_cpu_percent {
            warn!("平均 CPU 使用率 ({:.1}%) 高于阈值 ({:.1}%)", stats.avg_cpu, self.thresholds.max_cpu_percent);
        }
        if stats.avg_memory > self.thresholds.max_memory_mb {
            warn!("平均内存使用 ({:.1}MB) 高于阈值 ({:.1}MB)", stats.avg_memory, self.thresholds.max_memory_mb);
        }
    }
    
    /// 计算性能统计信息
    fn calculate_stats(&self) -> PerformanceStats {
        if self.performance_history.is_empty() {
            return PerformanceStats::default();
        }
        
        let current = &self.performance_history[self.performance_history.len() - 1];
        let count = self.performance_history.len() as f32;
        
        let mut fps_sum = 0.0;
        let mut cpu_sum = 0.0;
        let mut memory_sum = 0.0;
        let mut frame_time_sum = 0.0;
        let mut render_time_sum = 0.0;
        
        let mut min_fps = f32::INFINITY;
        let mut max_fps = f32::NEG_INFINITY;
        let mut max_cpu = f32::NEG_INFINITY;
        let mut max_memory = f32::NEG_INFINITY;
        let mut max_frame_time = f32::NEG_INFINITY;
        let mut max_render_time = f32::NEG_INFINITY;
        
        for snapshot in &self.performance_history {
            fps_sum += snapshot.fps;
            cpu_sum += snapshot.cpu_percent;
            memory_sum += snapshot.memory_mb;
            frame_time_sum += snapshot.frame_time_ms;
            render_time_sum += snapshot.render_time_ms;
            
            min_fps = min_fps.min(snapshot.fps);
            max_fps = max_fps.max(snapshot.fps);
            max_cpu = max_cpu.max(snapshot.cpu_percent);
            max_memory = max_memory.max(snapshot.memory_mb);
            max_frame_time = max_frame_time.max(snapshot.frame_time_ms);
            max_render_time = max_render_time.max(snapshot.render_time_ms);
        }
        
        PerformanceStats {
            current_fps: current.fps,
            avg_fps: fps_sum / count,
            min_fps,
            max_fps,
            current_cpu: current.cpu_percent,
            avg_cpu: cpu_sum / count,
            max_cpu,
            current_memory: current.memory_mb,
            avg_memory: memory_sum / count,
            max_memory,
            current_frame_time: current.frame_time_ms,
            avg_frame_time: frame_time_sum / count,
            max_frame_time,
            current_render_time: current.render_time_ms,
            avg_render_time: render_time_sum / count,
            max_render_time,
            sample_count: self.performance_history.len(),
        }
    }
    
    /// 获取当前性能统计信息
    pub fn get_stats(&self) -> PerformanceStats {
        self.calculate_stats()
    }
    
    /// 获取性能历史记录
    pub fn get_history(&self) -> &VecDeque<PerformanceSnapshot> {
        &self.performance_history
    }
    
    /// 更新性能阈值
    pub fn update_thresholds(&mut self, thresholds: PerformanceThresholds) {
        info!("更新性能阈值: {:?}", thresholds);
        self.thresholds = thresholds;
    }
    
    /// 重置性能历史记录
    pub fn reset_history(&mut self) {
        info!("重置性能历史记录");
        self.performance_history.clear();
        self.fps_calculator.reset();
    }
}

/// FPS 计算器
struct FpsCalculator {
    frame_times: VecDeque<Instant>,
    update_interval: Duration,
    last_fps: f32,
    last_update: Instant,
}

impl FpsCalculator {
    fn new(update_interval: Duration) -> Self {
        Self {
            frame_times: VecDeque::new(),
            update_interval,
            last_fps: 0.0,
            last_update: Instant::now(),
        }
    }
    
    fn update(&mut self) -> f32 {
        let now = Instant::now();
        self.frame_times.push_back(now);
        
        // 移除超过更新间隔的旧帧时间
        while let Some(&front_time) = self.frame_times.front() {
            if now.duration_since(front_time) > self.update_interval {
                self.frame_times.pop_front();
            } else {
                break;
            }
        }
        
        // 计算 FPS
        if self.frame_times.len() > 1 {
            let duration = now.duration_since(self.frame_times[0]);
            if duration.as_secs_f32() > 0.0 {
                self.last_fps = (self.frame_times.len() - 1) as f32 / duration.as_secs_f32();
            }
        }
        
        self.last_update = now;
        self.last_fps
    }
    
    fn reset(&mut self) {
        self.frame_times.clear();
        self.last_fps = 0.0;
        self.last_update = Instant::now();
    }
}

/// CPU 使用率监控器
struct CpuMonitor {
    last_check: Instant,
    check_interval: Duration,
    cached_usage: f32,
}

impl CpuMonitor {
    fn new() -> Self {
        Self {
            last_check: Instant::now(),
            check_interval: Duration::from_millis(500), // 每500ms检查一次
            cached_usage: 0.0,
        }
    }
    
    fn get_usage(&mut self) -> f32 {
        let now = Instant::now();
        
        if now.duration_since(self.last_check) >= self.check_interval {
            self.cached_usage = self.measure_cpu_usage();
            self.last_check = now;
        }
        
        self.cached_usage
    }
    
    #[cfg(target_os = "windows")]
    fn measure_cpu_usage(&self) -> f32 {
        // Windows 实现 - 简化版本
        // 在实际应用中，可以使用 Windows API 获取更精确的 CPU 使用率
        use std::process::Command;
        
        match Command::new("wmic")
            .args(&["cpu", "get", "loadpercentage", "/value"])
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.starts_with("LoadPercentage=") {
                        if let Ok(usage) = line.split('=').nth(1).unwrap_or("0").parse::<f32>() {
                            return usage;
                        }
                    }
                }
                15.0 // 默认值
            }
            Err(_) => 15.0, // 默认值
        }
    }
    
    #[cfg(target_os = "macos")]
    fn measure_cpu_usage(&self) -> f32 {
        // macOS 实现
        use std::process::Command;
        
        match Command::new("top")
            .args(&["-l", "1", "-n", "0"])
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("CPU usage:") {
                        // 解析 CPU 使用率
                        // 格式类似: "CPU usage: 12.34% user, 5.67% sys, 81.99% idle"
                        if let Some(user_part) = line.split("% user").next() {
                            if let Some(usage_str) = user_part.split_whitespace().last() {
                                if let Ok(usage) = usage_str.parse::<f32>() {
                                    return usage;
                                }
                            }
                        }
                    }
                }
                15.0 // 默认值
            }
            Err(_) => 15.0, // 默认值
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    fn measure_cpu_usage(&self) -> f32 {
        // Linux 和其他平台实现
        use std::fs;
        
        match fs::read_to_string("/proc/loadavg") {
            Ok(content) => {
                if let Some(load_str) = content.split_whitespace().next() {
                    if let Ok(load) = load_str.parse::<f32>() {
                        // 将负载转换为百分比（简化）
                        return (load * 100.0).min(100.0);
                    }
                }
                15.0 // 默认值
            }
            Err(_) => 15.0, // 默认值
        }
    }
}

/// 内存使用监控器
struct MemoryUsageMonitor {
    last_check: Instant,
    check_interval: Duration,
    cached_usage: f32,
}

impl MemoryUsageMonitor {
    fn new() -> Self {
        Self {
            last_check: Instant::now(),
            check_interval: Duration::from_millis(1000), // 每秒检查一次
            cached_usage: 0.0,
        }
    }
    
    fn get_usage_mb(&mut self) -> f32 {
        let now = Instant::now();
        
        if now.duration_since(self.last_check) >= self.check_interval {
            self.cached_usage = self.measure_memory_usage();
            self.last_check = now;
        }
        
        self.cached_usage
    }
    
    #[cfg(target_os = "windows")]
    fn measure_memory_usage(&self) -> f32 {
        use std::mem;
        use std::ptr;
        
        #[repr(C)]
        struct ProcessMemoryCounters {
            cb: u32,
            page_fault_count: u32,
            peak_working_set_size: usize,
            working_set_size: usize,
            quota_peak_paged_pool_usage: usize,
            quota_paged_pool_usage: usize,
            quota_peak_non_paged_pool_usage: usize,
            quota_non_paged_pool_usage: usize,
            pagefile_usage: usize,
            peak_pagefile_usage: usize,
        }
        
        extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn GetProcessMemoryInfo(
                process: *mut std::ffi::c_void,
                counters: *mut ProcessMemoryCounters,
                cb: u32,
            ) -> i32;
        }
        
        let mut counters = ProcessMemoryCounters {
            cb: mem::size_of::<ProcessMemoryCounters>() as u32,
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            pagefile_usage: 0,
            peak_pagefile_usage: 0,
        };
        
        unsafe {
            if GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters,
                mem::size_of::<ProcessMemoryCounters>() as u32,
            ) != 0
            {
                (counters.working_set_size as f32) / (1024.0 * 1024.0)
            } else {
                100.0 // 默认值
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    fn measure_memory_usage(&self) -> f32 {
        // 非 Windows 平台的简化实现
        100.0 // 默认值
    }
}

/// 性能警报
#[derive(Debug, Clone)]
pub enum PerformanceAlert {
    LowFps { current: f32, threshold: f32 },
    HighCpu { current: f32, threshold: f32 },
    HighMemory { current: f32, threshold: f32 },
    SlowFrame { current: f32, threshold: f32 },
    SlowRender { current: f32, threshold: f32 },
}

impl PerformanceAlert {
    /// 获取警报消息
    pub fn message(&self) -> String {
        match self {
            PerformanceAlert::LowFps { current, threshold } => {
                format!("FPS 过低: {:.1} < {:.1}", current, threshold)
            }
            PerformanceAlert::HighCpu { current, threshold } => {
                format!("CPU 使用率过高: {:.1}% > {:.1}%", current, threshold)
            }
            PerformanceAlert::HighMemory { current, threshold } => {
                format!("内存使用过高: {:.1}MB > {:.1}MB", current, threshold)
            }
            PerformanceAlert::SlowFrame { current, threshold } => {
                format!("帧时间过长: {:.1}ms > {:.1}ms", current, threshold)
            }
            PerformanceAlert::SlowRender { current, threshold } => {
                format!("渲染时间过长: {:.1}ms > {:.1}ms", current, threshold)
            }
        }
    }
    
    /// 获取警报严重程度
    pub fn severity(&self) -> AlertSeverity {
        match self {
            PerformanceAlert::LowFps { current, threshold } => {
                if current < threshold * 0.5 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                }
            }
            PerformanceAlert::HighCpu { current, threshold } => {
                if current > threshold * 2.0 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                }
            }
            PerformanceAlert::HighMemory { current, threshold } => {
                if current > threshold * 1.5 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                }
            }
            PerformanceAlert::SlowFrame { current, threshold } => {
                if current > threshold * 2.0 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                }
            }
            PerformanceAlert::SlowRender { current, threshold } => {
                if current > threshold * 2.0 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                }
            }
        }
    }
}

/// 警报严重程度
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

/// 性能统计信息
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    pub current_fps: f32,
    pub avg_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub current_cpu: f32,
    pub avg_cpu: f32,
    pub max_cpu: f32,
    pub current_memory: f32,
    pub avg_memory: f32,
    pub max_memory: f32,
    pub current_frame_time: f32,
    pub avg_frame_time: f32,
    pub max_frame_time: f32,
    pub current_render_time: f32,
    pub avg_render_time: f32,
    pub max_render_time: f32,
    pub sample_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new(
            100,
            Duration::from_secs(5),
            None,
        );
        
        let stats = monitor.get_stats();
        assert_eq!(stats.sample_count, 0);
    }
    
    #[test]
    fn test_fps_calculator() {
        let mut calculator = FpsCalculator::new(Duration::from_secs(1));
        
        // 模拟 60 FPS
        for _ in 0..60 {
            calculator.update();
            std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }
        
        let fps = calculator.update();
        assert!(fps > 50.0 && fps < 70.0); // 允许一些误差
    }
    
    #[test]
    fn test_performance_thresholds() {
        let thresholds = PerformanceThresholds::default();
        assert_eq!(thresholds.min_fps, 30.0);
        assert_eq!(thresholds.max_cpu_percent, 25.0);
        assert_eq!(thresholds.max_memory_mb, 200.0);
    }
    
    #[test]
    fn test_performance_alert_message() {
        let alert = PerformanceAlert::LowFps {
            current: 25.0,
            threshold: 30.0,
        };
        
        let message = alert.message();
        assert!(message.contains("FPS 过低"));
        assert!(message.contains("25.0"));
        assert!(message.contains("30.0"));
    }
    
    #[test]
    fn test_performance_alert_severity() {
        let warning_alert = PerformanceAlert::LowFps {
            current: 25.0,
            threshold: 30.0,
        };
        assert_eq!(warning_alert.severity(), AlertSeverity::Warning);
        
        let critical_alert = PerformanceAlert::LowFps {
            current: 10.0,
            threshold: 30.0,
        };
        assert_eq!(critical_alert.severity(), AlertSeverity::Critical);
    }
    
    #[test]
    fn test_cpu_monitor() {
        let mut monitor = CpuMonitor::new();
        let usage = monitor.get_usage();
        assert!(usage >= 0.0 && usage <= 100.0);
    }
    
    #[test]
    fn test_memory_monitor() {
        let mut monitor = MemoryUsageMonitor::new();
        let usage = monitor.get_usage_mb();
        assert!(usage > 0.0);
    }
    
    #[test]
    fn test_performance_stats_calculation() {
        let mut monitor = PerformanceMonitor::new(
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
    }
    
    #[test]
    fn test_performance_history_limit() {
        let mut monitor = PerformanceMonitor::new(
            3, // 最大历史记录数
            Duration::from_secs(1),
            None,
        );
        
        // 添加超过限制的记录
        for _ in 0..5 {
            monitor.record_frame(Duration::from_millis(16), Duration::from_millis(8));
        }
        
        assert!(monitor.get_history().len() <= 3);
    }
}