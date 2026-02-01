// 内存管理和优化模块

use log::{debug, info, warn};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 帧缓冲区池，用于复用内存分配
pub struct FrameBufferPool {
    /// 可用的缓冲区队列
    available_buffers: Arc<Mutex<VecDeque<Vec<u8>>>>,
    /// 缓冲区大小
    buffer_size: usize,
    /// 最大缓冲区数量
    max_buffers: usize,
    /// 当前分配的缓冲区数量
    allocated_count: Arc<Mutex<usize>>,
    /// 创建时间
    created_at: Instant,
}

impl FrameBufferPool {
    /// 创建新的帧缓冲区池
    pub fn new(buffer_size: usize, initial_count: usize, max_buffers: usize) -> Self {
        info!("创建帧缓冲区池: 缓冲区大小={}字节, 初始数量={}, 最大数量={}", 
              buffer_size, initial_count, max_buffers);
        
        let mut buffers = VecDeque::with_capacity(max_buffers);
        
        // 预分配初始缓冲区
        for _ in 0..initial_count {
            buffers.push_back(vec![0u8; buffer_size]);
        }
        
        debug!("预分配了 {} 个缓冲区", initial_count);
        
        Self {
            available_buffers: Arc::new(Mutex::new(buffers)),
            buffer_size,
            max_buffers,
            allocated_count: Arc::new(Mutex::new(initial_count)),
            created_at: Instant::now(),
        }
    }
    
    /// 获取一个缓冲区
    pub fn get_buffer(&self) -> Vec<u8> {
        let mut buffers = self.available_buffers.lock().unwrap();
        
        if let Some(mut buffer) = buffers.pop_front() {
            // 重用现有缓冲区
            buffer.clear();
            buffer.resize(self.buffer_size, 0);
            debug!("重用缓冲区，剩余可用: {}", buffers.len());
            buffer
        } else {
            // 检查是否可以分配新缓冲区
            let mut allocated = self.allocated_count.lock().unwrap();
            if *allocated < self.max_buffers {
                *allocated += 1;
                debug!("分配新缓冲区，总分配数: {}", *allocated);
                vec![0u8; self.buffer_size]
            } else {
                // 达到最大限制，创建临时缓冲区
                warn!("达到缓冲区最大限制 {}，创建临时缓冲区", self.max_buffers);
                vec![0u8; self.buffer_size]
            }
        }
    }
    
    /// 归还缓冲区到池中
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        if buffer.capacity() >= self.buffer_size {
            let mut buffers = self.available_buffers.lock().unwrap();
            if buffers.len() < self.max_buffers {
                buffers.push_back(buffer);
                debug!("归还缓冲区到池中，可用数量: {}", buffers.len());
            } else {
                debug!("缓冲区池已满，丢弃缓冲区");
            }
        } else {
            debug!("缓冲区容量不足，丢弃");
        }
    }
    
    /// 获取池统计信息
    pub fn get_stats(&self) -> PoolStats {
        let buffers = self.available_buffers.lock().unwrap();
        let allocated = self.allocated_count.lock().unwrap();
        
        PoolStats {
            available_count: buffers.len(),
            allocated_count: *allocated,
            max_buffers: self.max_buffers,
            buffer_size: self.buffer_size,
            uptime: self.created_at.elapsed(),
        }
    }
    
    /// 清理未使用的缓冲区
    pub fn cleanup_unused(&self) {
        let mut buffers = self.available_buffers.lock().unwrap();
        let mut allocated = self.allocated_count.lock().unwrap();
        
        // 保留一半的缓冲区，释放其余的
        let keep_count = (buffers.len() / 2).max(1);
        let remove_count = buffers.len().saturating_sub(keep_count);
        
        for _ in 0..remove_count {
            buffers.pop_back();
            *allocated = allocated.saturating_sub(1);
        }
        
        if remove_count > 0 {
            info!("清理了 {} 个未使用的缓冲区，剩余: {}", remove_count, buffers.len());
        }
    }
}

/// 缓冲区池统计信息
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available_count: usize,
    pub allocated_count: usize,
    pub max_buffers: usize,
    pub buffer_size: usize,
    pub uptime: Duration,
}

/// 内存使用监控器
pub struct MemoryMonitor {
    /// 上次检查时间
    last_check: Instant,
    /// 检查间隔
    check_interval: Duration,
    /// 内存使用历史
    memory_history: VecDeque<MemorySnapshot>,
    /// 最大历史记录数
    max_history: usize,
    /// 内存泄漏检测阈值（MB）
    leak_threshold_mb: f32,
}

/// 内存快照
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub memory_mb: f32,
    pub allocated_buffers: usize,
}

impl MemoryMonitor {
    /// 创建新的内存监控器
    pub fn new(check_interval: Duration, max_history: usize, leak_threshold_mb: f32) -> Self {
        info!("创建内存监控器: 检查间隔={:?}, 最大历史={}, 泄漏阈值={}MB", 
              check_interval, max_history, leak_threshold_mb);
        
        Self {
            last_check: Instant::now(),
            check_interval,
            memory_history: VecDeque::with_capacity(max_history),
            max_history,
            leak_threshold_mb,
        }
    }
    
    /// 更新内存监控
    pub fn update(&mut self, current_memory_mb: f32, allocated_buffers: usize) -> Option<MemoryAlert> {
        let now = Instant::now();
        
        if now.duration_since(self.last_check) >= self.check_interval {
            self.last_check = now;
            
            // 记录当前快照
            let snapshot = MemorySnapshot {
                timestamp: now,
                memory_mb: current_memory_mb,
                allocated_buffers,
            };
            
            self.memory_history.push_back(snapshot.clone());
            
            // 保持历史记录在限制内
            while self.memory_history.len() > self.max_history {
                self.memory_history.pop_front();
            }
            
            debug!("内存监控更新: {:.1}MB, 缓冲区: {}", current_memory_mb, allocated_buffers);
            
            // 检查内存泄漏
            self.check_memory_leak()
        } else {
            None
        }
    }
    
    /// 检查内存泄漏
    fn check_memory_leak(&self) -> Option<MemoryAlert> {
        if self.memory_history.len() < 3 {
            return None;
        }
        
        // 获取最近的几个快照
        let recent_snapshots: Vec<_> = self.memory_history.iter().rev().take(3).collect();
        
        // 检查内存是否持续增长
        let mut is_increasing = true;
        let mut max_increase = 0.0f32;
        
        for i in 1..recent_snapshots.len() {
            let current = recent_snapshots[i - 1].memory_mb;
            let previous = recent_snapshots[i].memory_mb;
            let increase = current - previous;
            
            if increase <= 0.0 {
                is_increasing = false;
                break;
            }
            
            max_increase = max_increase.max(increase);
        }
        
        if is_increasing && max_increase > self.leak_threshold_mb {
            warn!("检测到可能的内存泄漏: 连续增长 {:.1}MB", max_increase);
            Some(MemoryAlert::PossibleLeak {
                increase_mb: max_increase,
                current_mb: recent_snapshots[0].memory_mb,
            })
        } else {
            None
        }
    }
    
    /// 获取内存统计信息
    pub fn get_stats(&self) -> MemoryStats {
        if self.memory_history.is_empty() {
            return MemoryStats::default();
        }
        
        let current = &self.memory_history[self.memory_history.len() - 1];
        let min_memory = self.memory_history.iter().map(|s| s.memory_mb).fold(f32::INFINITY, f32::min);
        let max_memory = self.memory_history.iter().map(|s| s.memory_mb).fold(f32::NEG_INFINITY, f32::max);
        let avg_memory = self.memory_history.iter().map(|s| s.memory_mb).sum::<f32>() / self.memory_history.len() as f32;
        
        MemoryStats {
            current_mb: current.memory_mb,
            min_mb: min_memory,
            max_mb: max_memory,
            avg_mb: avg_memory,
            allocated_buffers: current.allocated_buffers,
            history_count: self.memory_history.len(),
        }
    }
}

/// 内存警报
#[derive(Debug, Clone)]
pub enum MemoryAlert {
    PossibleLeak {
        increase_mb: f32,
        current_mb: f32,
    },
    HighUsage {
        current_mb: f32,
        threshold_mb: f32,
    },
}

/// 内存统计信息
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub current_mb: f32,
    pub min_mb: f32,
    pub max_mb: f32,
    pub avg_mb: f32,
    pub allocated_buffers: usize,
    pub history_count: usize,
}

/// GPU 纹理管理器
pub struct TextureManager {
    /// 纹理缓存
    texture_cache: Vec<Option<wgpu::Texture>>,
    /// 最大缓存纹理数量
    max_cached_textures: usize,
    /// 纹理使用计数
    texture_usage: Vec<u32>,
    /// 上次清理时间
    last_cleanup: Instant,
    /// 清理间隔
    cleanup_interval: Duration,
}

impl TextureManager {
    /// 创建新的纹理管理器
    pub fn new(max_cached_textures: usize, cleanup_interval: Duration) -> Self {
        info!("创建纹理管理器: 最大缓存={}, 清理间隔={:?}", 
              max_cached_textures, cleanup_interval);
        
        Self {
            texture_cache: vec![None; max_cached_textures],
            max_cached_textures,
            texture_usage: vec![0; max_cached_textures],
            last_cleanup: Instant::now(),
            cleanup_interval,
        }
    }
    
    /// 获取或创建纹理
    pub fn get_or_create_texture(
        &mut self,
        device: &wgpu::Device,
        descriptor: &wgpu::TextureDescriptor,
    ) -> Option<&wgpu::Texture> {
        // 查找匹配的缓存纹理
        for (i, cached_texture) in self.texture_cache.iter().enumerate() {
            if let Some(texture) = cached_texture {
                if self.texture_matches_descriptor(texture, descriptor) {
                    self.texture_usage[i] += 1;
                    debug!("重用缓存纹理 {}, 使用次数: {}", i, self.texture_usage[i]);
                    return Some(texture);
                }
            }
        }
        
        // 查找空闲槽位
        for (i, cached_texture) in self.texture_cache.iter_mut().enumerate() {
            if cached_texture.is_none() {
                let new_texture = device.create_texture(descriptor);
                *cached_texture = Some(new_texture);
                self.texture_usage[i] = 1;
                debug!("创建新纹理并缓存到槽位 {}", i);
                return cached_texture.as_ref();
            }
        }
        
        // 缓存已满，替换使用次数最少的纹理
        let min_usage_index = self.texture_usage.iter()
            .enumerate()
            .min_by_key(|(_, &usage)| usage)
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        let new_texture = device.create_texture(descriptor);
        self.texture_cache[min_usage_index] = Some(new_texture);
        self.texture_usage[min_usage_index] = 1;
        
        warn!("纹理缓存已满，替换槽位 {} (使用次数: {})", 
              min_usage_index, self.texture_usage[min_usage_index]);
        
        self.texture_cache[min_usage_index].as_ref()
    }
    
    /// 检查纹理是否匹配描述符
    fn texture_matches_descriptor(&self, texture: &wgpu::Texture, descriptor: &wgpu::TextureDescriptor) -> bool {
        let size = texture.size();
        size.width == descriptor.size.width &&
        size.height == descriptor.size.height &&
        size.depth_or_array_layers == descriptor.size.depth_or_array_layers &&
        texture.format() == descriptor.format &&
        texture.usage() == descriptor.usage
    }
    
    /// 清理未使用的纹理
    pub fn cleanup_unused(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_cleanup) < self.cleanup_interval {
            return;
        }
        
        self.last_cleanup = now;
        
        let mut cleaned_count = 0;
        for (i, (texture, usage)) in self.texture_cache.iter_mut().zip(self.texture_usage.iter_mut()).enumerate() {
            if *usage == 0 && texture.is_some() {
                *texture = None;
                cleaned_count += 1;
                debug!("清理未使用的纹理槽位 {}", i);
            } else if *usage > 0 {
                // 减少使用计数，为下次清理做准备
                *usage = (*usage).saturating_sub(1);
            }
        }
        
        if cleaned_count > 0 {
            info!("清理了 {} 个未使用的纹理", cleaned_count);
        }
    }
    
    /// 获取纹理管理器统计信息
    pub fn get_stats(&self) -> TextureManagerStats {
        let cached_count = self.texture_cache.iter().filter(|t| t.is_some()).count();
        let total_usage: u32 = self.texture_usage.iter().sum();
        
        TextureManagerStats {
            cached_textures: cached_count,
            max_cached_textures: self.max_cached_textures,
            total_usage,
            cache_hit_rate: if total_usage > 0 { 
                (total_usage as f32 - cached_count as f32) / total_usage as f32 
            } else { 
                0.0 
            },
        }
    }
}

/// 纹理管理器统计信息
#[derive(Debug, Clone)]
pub struct TextureManagerStats {
    pub cached_textures: usize,
    pub max_cached_textures: usize,
    pub total_usage: u32,
    pub cache_hit_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_buffer_pool_creation() {
        let pool = FrameBufferPool::new(1024, 2, 5);
        let stats = pool.get_stats();
        
        assert_eq!(stats.buffer_size, 1024);
        assert_eq!(stats.max_buffers, 5);
        assert_eq!(stats.allocated_count, 2);
        assert_eq!(stats.available_count, 2);
    }
    
    #[test]
    fn test_frame_buffer_pool_get_return() {
        let pool = FrameBufferPool::new(1024, 1, 3);
        
        // 获取缓冲区
        let buffer1 = pool.get_buffer();
        assert_eq!(buffer1.len(), 1024);
        
        let stats = pool.get_stats();
        assert_eq!(stats.available_count, 0); // 应该被取走了
        
        // 归还缓冲区
        pool.return_buffer(buffer1);
        
        let stats = pool.get_stats();
        assert_eq!(stats.available_count, 1); // 应该被归还了
    }
    
    #[test]
    fn test_frame_buffer_pool_max_limit() {
        let pool = FrameBufferPool::new(1024, 0, 2);
        
        // 获取超过最大限制的缓冲区
        let _buffer1 = pool.get_buffer();
        let _buffer2 = pool.get_buffer();
        let buffer3 = pool.get_buffer(); // 这个应该是临时缓冲区
        
        assert_eq!(buffer3.len(), 1024);
        
        let stats = pool.get_stats();
        assert_eq!(stats.allocated_count, 2); // 不应该超过最大限制
    }
    
    #[test]
    fn test_memory_monitor_creation() {
        let monitor = MemoryMonitor::new(
            Duration::from_secs(1),
            10,
            50.0
        );
        
        let stats = monitor.get_stats();
        assert_eq!(stats.history_count, 0);
    }
    
    #[test]
    fn test_memory_monitor_update() {
        let mut monitor = MemoryMonitor::new(
            Duration::from_millis(100),
            5,
            10.0
        );
        
        // 第一次更新应该立即执行
        let alert = monitor.update(100.0, 5);
        assert!(alert.is_none()); // 没有足够的历史数据
        
        let stats = monitor.get_stats();
        assert_eq!(stats.current_mb, 100.0);
        assert_eq!(stats.history_count, 1);
    }
    
    #[test]
    fn test_memory_leak_detection() {
        let mut monitor = MemoryMonitor::new(
            Duration::from_millis(1), // 很短的间隔用于测试
            10,
            5.0 // 低阈值用于测试
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
            Some(MemoryAlert::PossibleLeak { increase_mb, current_mb }) => {
                assert!(increase_mb > 0.0);
                assert_eq!(current_mb, 120.0);
            }
            _ => panic!("应该检测到内存泄漏"),
        }
    }
    
    #[test]
    fn test_texture_manager_creation() {
        let manager = TextureManager::new(5, Duration::from_secs(60));
        let stats = manager.get_stats();
        
        assert_eq!(stats.cached_textures, 0);
        assert_eq!(stats.max_cached_textures, 5);
        assert_eq!(stats.total_usage, 0);
    }
    
    #[test]
    fn test_pool_cleanup() {
        let pool = FrameBufferPool::new(1024, 4, 10);
        
        // 获取一些缓冲区但不归还
        let _buffer1 = pool.get_buffer();
        let _buffer2 = pool.get_buffer();
        
        let stats_before = pool.get_stats();
        assert_eq!(stats_before.available_count, 2);
        
        // 清理未使用的缓冲区
        pool.cleanup_unused();
        
        let stats_after = pool.get_stats();
        assert!(stats_after.available_count <= stats_before.available_count);
    }
    
    #[test]
    fn test_memory_stats_calculation() {
        let mut monitor = MemoryMonitor::new(
            Duration::from_millis(1),
            10,
            10.0
        );
        
        // 添加一些内存快照
        std::thread::sleep(Duration::from_millis(2));
        monitor.update(100.0, 5);
        
        std::thread::sleep(Duration::from_millis(2));
        monitor.update(150.0, 6);
        
        std::thread::sleep(Duration::from_millis(2));
        monitor.update(120.0, 5);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.current_mb, 120.0);
        assert_eq!(stats.min_mb, 100.0);
        assert_eq!(stats.max_mb, 150.0);
        assert!((stats.avg_mb - 123.33).abs() < 0.1); // 平均值
        assert_eq!(stats.history_count, 3);
    }
}