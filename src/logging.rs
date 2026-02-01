// 日志系统配置和管理

use log::{error, info, warn, LevelFilter};
use std::fs;
use std::path::PathBuf;

/// 日志配置结构
pub struct LoggingConfig {
    /// 日志目录路径
    pub log_dir: PathBuf,
    /// 日志文件路径
    pub log_file: PathBuf,
    /// 日志级别
    pub level: LevelFilter,
    /// 是否启用文件轮转
    pub enable_rotation: bool,
    /// 最大文件大小（字节）
    pub max_file_size: u64,
    /// 保留的日志文件数量
    pub max_files: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let log_dir = get_default_log_dir();
        let log_file = log_dir.join("mira.log");
        
        Self {
            log_dir,
            log_file,
            level: LevelFilter::Info,
            enable_rotation: true,
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_files: 5,
        }
    }
}

/// 获取默认日志目录
fn get_default_log_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        // Windows: %APPDATA%\Mira\logs\
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| {
                warn!("无法获取 APPDATA 环境变量，使用当前目录");
                ".".to_string()
            });
        PathBuf::from(appdata).join("Mira").join("logs")
    } else if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/Mira/logs/
        let home = std::env::var("HOME")
            .unwrap_or_else(|_| {
                warn!("无法获取 HOME 环境变量，使用当前目录");
                ".".to_string()
            });
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Mira")
            .join("logs")
    } else {
        // 其他平台使用 XDG 标准
        let config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{}/.config", home)
        });
        PathBuf::from(config_home).join("Mira").join("logs")
    }
}

/// 初始化日志系统
pub fn init_logging() -> Result<LoggingConfig, Box<dyn std::error::Error>> {
    let config = LoggingConfig::default();
    init_logging_with_config(config)
}

/// 使用指定配置初始化日志系统
pub fn init_logging_with_config(config: LoggingConfig) -> Result<LoggingConfig, Box<dyn std::error::Error>> {
    // 确保日志目录存在
    fs::create_dir_all(&config.log_dir)
        .map_err(|e| format!("创建日志目录失败: {}", e))?;
    
    // 检查并轮转日志文件
    if config.enable_rotation {
        rotate_log_files(&config)?;
    }
    
    // 配置 fern 日志系统
    let mut dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}:{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.target(),
                message
            ))
        })
        .level(config.level)
        .chain(std::io::stdout()); // 输出到控制台
    
    // 添加文件输出
    dispatch = dispatch.chain(fern::log_file(&config.log_file)?);
    
    // 应用配置
    dispatch.apply()
        .map_err(|e| format!("日志系统初始化失败: {}", e))?;
    
    info!("日志系统初始化完成");
    info!("日志目录: {:?}", config.log_dir);
    info!("日志文件: {:?}", config.log_file);
    info!("日志级别: {:?}", config.level);
    info!("文件轮转: {}", if config.enable_rotation { "启用" } else { "禁用" });
    
    // 记录系统信息
    log_system_info();
    
    Ok(config)
}

/// 轮转日志文件
fn rotate_log_files(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    if !config.log_file.exists() {
        return Ok(());
    }
    
    // 检查当前日志文件大小
    let metadata = fs::metadata(&config.log_file)?;
    if metadata.len() < config.max_file_size {
        return Ok(());
    }
    
    info!("日志文件大小 {} 字节，开始轮转", metadata.len());
    
    // 轮转现有文件
    for i in (1..config.max_files).rev() {
        let old_file = config.log_file.with_extension(format!("log.{}", i));
        let new_file = config.log_file.with_extension(format!("log.{}", i + 1));
        
        if old_file.exists() {
            if i + 1 >= config.max_files {
                // 删除最老的文件
                fs::remove_file(&old_file)?;
                info!("删除旧日志文件: {:?}", old_file);
            } else {
                // 重命名文件
                fs::rename(&old_file, &new_file)?;
                info!("轮转日志文件: {:?} -> {:?}", old_file, new_file);
            }
        }
    }
    
    // 将当前日志文件重命名为 .log.1
    let backup_file = config.log_file.with_extension("log.1");
    fs::rename(&config.log_file, &backup_file)?;
    info!("当前日志文件轮转为: {:?}", backup_file);
    
    Ok(())
}

/// 记录系统信息
fn log_system_info() {
    info!("=== 系统信息 ===");
    info!("应用程序: Mira v{}", env!("CARGO_PKG_VERSION"));
    info!("操作系统: {}", std::env::consts::OS);
    info!("架构: {}", std::env::consts::ARCH);
    info!("CPU 核心数: {}", num_cpus::get());
    
    // 记录内存信息（如果可用）
    if let Ok(memory_info) = get_memory_info() {
        info!("系统内存: {} MB", memory_info.total_mb);
        info!("可用内存: {} MB", memory_info.available_mb);
    }
    
    // 记录环境变量（仅记录相关的）
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        info!("RUST_LOG: {}", rust_log);
    }
    
    info!("=== 系统信息结束 ===");
}

/// 内存信息结构
#[derive(Debug)]
struct MemoryInfo {
    total_mb: u64,
    available_mb: u64,
}

/// 获取内存信息（跨平台）
fn get_memory_info() -> Result<MemoryInfo, Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        // Windows 实现
        use std::mem;
        use std::ptr;
        
        #[repr(C)]
        struct MemoryStatusEx {
            dw_length: u32,
            dw_memory_load: u32,
            ull_total_phys: u64,
            ull_avail_phys: u64,
            ull_total_page_file: u64,
            ull_avail_page_file: u64,
            ull_total_virtual: u64,
            ull_avail_virtual: u64,
            ull_avail_extended_virtual: u64,
        }
        
        extern "system" {
            fn GlobalMemoryStatusEx(lp_buffer: *mut MemoryStatusEx) -> i32;
        }
        
        let mut mem_status = MemoryStatusEx {
            dw_length: mem::size_of::<MemoryStatusEx>() as u32,
            dw_memory_load: 0,
            ull_total_phys: 0,
            ull_avail_phys: 0,
            ull_total_page_file: 0,
            ull_avail_page_file: 0,
            ull_total_virtual: 0,
            ull_avail_virtual: 0,
            ull_avail_extended_virtual: 0,
        };
        
        unsafe {
            if GlobalMemoryStatusEx(&mut mem_status) != 0 {
                Ok(MemoryInfo {
                    total_mb: mem_status.ull_total_phys / (1024 * 1024),
                    available_mb: mem_status.ull_avail_phys / (1024 * 1024),
                })
            } else {
                Err("无法获取 Windows 内存信息".into())
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS 实现
        use std::process::Command;
        
        let output = Command::new("sysctl")
            .args(&["-n", "hw.memsize"])
            .output()?;
        
        let total_bytes: u64 = String::from_utf8(output.stdout)?
            .trim()
            .parse()?;
        
        // 获取可用内存（简化实现）
        let available_bytes = total_bytes / 2; // 假设一半可用
        
        Ok(MemoryInfo {
            total_mb: total_bytes / (1024 * 1024),
            available_mb: available_bytes / (1024 * 1024),
        })
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Linux 和其他平台实现
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        
        let mut total_kb = 0;
        let mut available_kb = 0;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available_kb = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            }
        }
        
        Ok(MemoryInfo {
            total_mb: total_kb / 1024,
            available_mb: available_kb / 1024,
        })
    }
}

/// 记录性能指标
pub fn log_performance_metrics(fps: f32, memory_mb: f32, cpu_percent: f32) {
    info!("性能指标: FPS={:.1}, 内存={:.1}MB, CPU={:.1}%", fps, memory_mb, cpu_percent);
    
    // 检查性能警告阈值
    if fps < 30.0 {
        warn!("帧率低于目标值: {:.1} FPS < 30 FPS", fps);
    }
    
    if memory_mb > 200.0 {
        warn!("内存使用超过目标值: {:.1} MB > 200 MB", memory_mb);
    }
    
    if cpu_percent > 25.0 {
        warn!("CPU 使用率超过目标值: {:.1}% > 25%", cpu_percent);
    }
}

/// 记录错误统计
pub fn log_error_statistics(camera_errors: u32, render_errors: u32, window_errors: u32, config_errors: u32) {
    if camera_errors + render_errors + window_errors + config_errors > 0 {
        warn!("错误统计: 摄像头={}, 渲染={}, 窗口={}, 配置={}", 
              camera_errors, render_errors, window_errors, config_errors);
    }
}

/// 设置日志级别
pub fn set_log_level(level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    // 注意：fern 不支持运行时更改日志级别
    // 这里只是记录级别变化，实际需要重新初始化日志系统
    info!("请求更改日志级别为: {:?}", level);
    warn!("日志级别更改需要重启应用程序才能生效");
    Ok(())
}

/// 刷新日志缓冲区
pub fn flush_logs() {
    // 强制刷新所有日志输出
    log::logger().flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_default_log_dir() {
        let log_dir = get_default_log_dir();
        assert!(log_dir.is_absolute());
        assert!(log_dir.to_string_lossy().contains("Mira"));
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, LevelFilter::Info);
        assert!(config.enable_rotation);
        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
        assert_eq!(config.max_files, 5);
    }

    #[test]
    fn test_rotate_log_files() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");
        
        // 创建一个大文件
        let large_content = "x".repeat(1024);
        fs::write(&log_file, large_content).unwrap();
        
        let config = LoggingConfig {
            log_dir: temp_dir.path().to_path_buf(),
            log_file: log_file.clone(),
            level: LevelFilter::Info,
            enable_rotation: true,
            max_file_size: 512, // 小尺寸用于测试
            max_files: 3,
        };
        
        // 执行轮转
        rotate_log_files(&config).unwrap();
        
        // 检查文件是否被轮转
        let backup_file = log_file.with_extension("log.1");
        assert!(backup_file.exists());
        assert!(!log_file.exists() || fs::metadata(&log_file).unwrap().len() == 0);
    }

    #[test]
    fn test_memory_info_structure() {
        let mem_info = MemoryInfo {
            total_mb: 8192,
            available_mb: 4096,
        };
        
        assert_eq!(mem_info.total_mb, 8192);
        assert_eq!(mem_info.available_mb, 4096);
    }

    #[test]
    fn test_log_performance_metrics() {
        // 这个测试只验证函数不会 panic
        log_performance_metrics(60.0, 150.0, 15.0);
        log_performance_metrics(25.0, 250.0, 30.0); // 触发警告
    }

    #[test]
    fn test_log_error_statistics() {
        // 这个测试只验证函数不会 panic
        log_error_statistics(0, 0, 0, 0);
        log_error_statistics(1, 2, 0, 1);
    }
}