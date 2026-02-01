// 配置管理器实现

use crate::error::ConfigError;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// 配置版本，用于迁移
    #[serde(default = "default_version")]
    pub version: String,
    pub window: WindowConfig,
    pub camera: CameraConfig,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowConfig {
    pub position_x: f64,
    pub position_y: f64,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
    pub shape: String,
}

/// 摄像头配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CameraConfig {
    pub device_index: usize,
}

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path()?;
        let config = Self::default_config();

        Ok(Self {
            config_path,
            config,
        })
    }

    /// 获取平台特定的配置文件路径
    fn get_config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = if cfg!(target_os = "windows") {
            // Windows: %APPDATA%\Mira\config.toml
            std::env::var("APPDATA")
                .map_err(|_| ConfigError::WriteError("无法获取 APPDATA 环境变量".to_string()))?
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support/Mira/config.toml
            let home = std::env::var("HOME")
                .map_err(|_| ConfigError::WriteError("无法获取 HOME 环境变量".to_string()))?;
            format!("{}/Library/Application Support", home)
        } else {
            // 其他平台使用 XDG 标准
            std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.config", home)
            })
        };

        let mut path = PathBuf::from(config_dir);
        path.push("Mira");
        path.push("config.toml");
        Ok(path)
    }

    /// 创建默认配置
    fn default_config() -> AppConfig {
        AppConfig {
            version: "1.0".to_string(),
            window: WindowConfig {
                position_x: 100.0,
                position_y: 100.0,
                width: 400,
                height: 400,
                rotation: 0.0,
                shape: "Circle".to_string(),
            },
            camera: CameraConfig { device_index: 0 },
        }
    }

    /// 确保配置目录存在
    fn ensure_config_dir(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::WriteError(format!("无法创建配置目录: {}", e)))?;
            }
        }
        Ok(())
    }

    /// 加载配置
    pub fn load(&mut self) -> Result<AppConfig, ConfigError> {
        if !self.config_path.exists() {
            // 配置文件不存在，创建默认配置
            info!("配置文件不存在，创建默认配置: {:?}", self.config_path);
            let default_config = Self::default_config();
            self.save(&default_config)?;
            self.config = default_config.clone();
            return Ok(default_config);
        }

        // 读取配置文件
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| ConfigError::ParseError(format!("无法读取配置文件: {}", e)))?;

        // 解析 TOML
        match toml::from_str::<AppConfig>(&content) {
            Ok(mut config) => {
                // 检查是否需要迁移
                if self.needs_migration(&config) {
                    config = self.migrate_config(config)?;
                }
                
                // 验证和修正配置
                self.validate_and_fix_config(&mut config);
                self.config = config.clone();
                Ok(config)
            }
            Err(e) => {
                // 配置文件损坏，备份并使用默认配置
                warn!("配置文件解析失败: {}，使用默认配置", e);
                self.backup_corrupted_config()?;
                let default_config = Self::default_config();
                self.save(&default_config)?;
                self.config = default_config.clone();
                Ok(default_config)
            }
        }
    }

    /// 保存配置
    pub fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        // 确保配置目录存在
        self.ensure_config_dir()?;

        // 验证配置
        let mut validated_config = config.clone();
        self.validate_and_fix_config(&mut validated_config);

        // 序列化为 TOML
        let content = toml::to_string_pretty(&validated_config)
            .map_err(|e| ConfigError::WriteError(format!("配置序列化失败: {}", e)))?;

        // 写入文件
        fs::write(&self.config_path, content)
            .map_err(|e| ConfigError::WriteError(format!("无法写入配置文件: {}", e)))?;

        info!("配置已保存到: {:?}", self.config_path);
        Ok(())
    }

    /// 备份损坏的配置文件
    fn backup_corrupted_config(&self) -> Result<(), ConfigError> {
        let backup_path = self.config_path.with_extension("toml.backup");
        if let Err(e) = fs::copy(&self.config_path, &backup_path) {
            warn!("无法备份损坏的配置文件: {}", e);
        } else {
            info!("损坏的配置文件已备份到: {:?}", backup_path);
        }
        Ok(())
    }

    /// 检查配置是否需要迁移
    fn needs_migration(&self, config: &AppConfig) -> bool {
        // 当前版本是 1.0，如果配置版本不同则需要迁移
        config.version != "1.0"
    }

    /// 迁移配置到当前版本
    fn migrate_config(&self, mut config: AppConfig) -> Result<AppConfig, ConfigError> {
        let old_version = config.version.clone();
        
        match config.version.as_str() {
            // 如果是空版本或旧版本，迁移到 1.0
            "" | "0.1" | "0.9" => {
                info!("迁移配置从版本 {} 到 1.0", old_version);
                config.version = "1.0".to_string();
                
                // 在这里可以添加特定的迁移逻辑
                // 例如：重命名字段、添加默认值等
                
                // 保存迁移后的配置
                self.save(&config)?;
            }
            // 如果版本更新，使用默认配置
            version if version > "1.0" => {
                warn!("配置版本 {} 比当前支持的版本 1.0 更新，使用默认配置", version);
                config = Self::default_config();
                self.save(&config)?;
            }
            // 版本匹配，无需迁移
            "1.0" => {}
            // 未知版本，使用默认配置
            _ => {
                warn!("未知配置版本 {}，使用默认配置", old_version);
                config = Self::default_config();
                self.save(&config)?;
            }
        }
        
        Ok(config)
    }

    /// 验证和修正配置值
    fn validate_and_fix_config(&self, config: &mut AppConfig) {
        // 确保版本正确
        if config.version != "1.0" {
            config.version = "1.0".to_string();
        }

        // 验证窗口位置（确保不是 NaN 或无穷大）
        if !config.window.position_x.is_finite() {
            warn!("窗口 X 位置无效，修正为 100.0");
            config.window.position_x = 100.0;
        }
        if !config.window.position_y.is_finite() {
            warn!("窗口 Y 位置无效，修正为 100.0");
            config.window.position_y = 100.0;
        }

        // 验证窗口尺寸（最小 100x100，最大 4096x4096）
        if config.window.width < 100 {
            warn!("窗口宽度 {} 小于最小值，修正为 100", config.window.width);
            config.window.width = 100;
        } else if config.window.width > 4096 {
            warn!("窗口宽度 {} 超过最大值，修正为 4096", config.window.width);
            config.window.width = 4096;
        }
        
        if config.window.height < 100 {
            warn!("窗口高度 {} 小于最小值，修正为 100", config.window.height);
            config.window.height = 100;
        } else if config.window.height > 4096 {
            warn!("窗口高度 {} 超过最大值，修正为 4096", config.window.height);
            config.window.height = 4096;
        }

        // 验证旋转角度（0-360度，处理 NaN 和无穷大）
        if !config.window.rotation.is_finite() {
            warn!("旋转角度无效，修正为 0.0");
            config.window.rotation = 0.0;
        } else if config.window.rotation < 0.0 || config.window.rotation >= 360.0 {
            let normalized = config.window.rotation % 360.0;
            let normalized = if normalized < 0.0 { normalized + 360.0 } else { normalized };
            warn!("旋转角度 {} 超出范围，修正为 {}", config.window.rotation, normalized);
            config.window.rotation = normalized;
        }

        // 验证形状名称
        let valid_shapes = ["Circle", "Ellipse", "Rectangle", "RoundedRectangle", "Heart"];
        if config.window.shape.is_empty() || !valid_shapes.contains(&config.window.shape.as_str()) {
            warn!("无效的形状名称 '{}'，修正为 Circle", config.window.shape);
            config.window.shape = "Circle".to_string();
        }

        // 验证摄像头设备索引（确保在合理范围内）
        if config.camera.device_index > 99 {
            warn!("摄像头设备索引 {} 过大，修正为 0", config.camera.device_index);
            config.camera.device_index = 0;
        }
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> &std::path::Path {
        &self.config_path
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_config_manager_creation() {
        let result = ConfigManager::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_config() {
        let manager = ConfigManager::new().unwrap();
        let config = manager.get_config();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.window.width, 400);
        assert_eq!(config.window.height, 400);
        assert_eq!(config.camera.device_index, 0);
        assert_eq!(config.window.shape, "Circle");
    }

    #[test]
    fn test_config_path_generation() {
        let result = ConfigManager::get_config_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("Mira"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_config_validation() {
        let mut manager = ConfigManager::new().unwrap();
        let mut config = AppConfig {
            version: "1.0".to_string(),
            window: WindowConfig {
                position_x: 100.0,
                position_y: 100.0,
                width: 50, // 小于最小值
                height: 50, // 小于最小值
                rotation: 400.0, // 超出范围
                shape: "InvalidShape".to_string(), // 无效形状
            },
            camera: CameraConfig { device_index: 0 },
        };

        manager.validate_and_fix_config(&mut config);

        assert_eq!(config.window.width, 100);
        assert_eq!(config.window.height, 100);
        assert!(config.window.rotation >= 0.0 && config.window.rotation < 360.0);
        assert_eq!(config.window.shape, "Circle");
    }

    #[test]
    fn test_rotation_normalization() {
        let manager = ConfigManager::new().unwrap();
        let mut config = AppConfig {
            version: "1.0".to_string(),
            window: WindowConfig {
                position_x: 100.0,
                position_y: 100.0,
                width: 400,
                height: 400,
                rotation: -45.0, // 负角度
                shape: "Circle".to_string(),
            },
            camera: CameraConfig { device_index: 0 },
        };

        manager.validate_and_fix_config(&mut config);
        assert_eq!(config.window.rotation, 315.0); // -45 + 360 = 315

        config.window.rotation = 450.0; // 超过 360
        manager.validate_and_fix_config(&mut config);
        assert_eq!(config.window.rotation, 90.0); // 450 % 360 = 90
    }

    #[test]
    fn test_save_and_load_config() {
        // 创建临时配置文件路径
        let temp_dir = std::env::temp_dir();
        let temp_config_path = temp_dir.join("test_mira_config.toml");
        
        // 确保测试开始时文件不存在
        let _ = fs::remove_file(&temp_config_path);

        // 创建配置管理器并设置临时路径
        let mut manager = ConfigManager::new().unwrap();
        manager.config_path = temp_config_path.clone();

        // 创建测试配置
        let test_config = AppConfig {
            version: "1.0".to_string(),
            window: WindowConfig {
                position_x: 200.0,
                position_y: 300.0,
                width: 500,
                height: 600,
                rotation: 45.0,
                shape: "Heart".to_string(),
            },
            camera: CameraConfig { device_index: 1 },
        };

        // 保存配置
        let save_result = manager.save(&test_config);
        assert!(save_result.is_ok());
        assert!(temp_config_path.exists());

        // 加载配置
        let load_result = manager.load();
        assert!(load_result.is_ok());
        let loaded_config = load_result.unwrap();

        // 验证配置内容
        assert_eq!(loaded_config.version, "1.0");
        assert_eq!(loaded_config.window.position_x, 200.0);
        assert_eq!(loaded_config.window.position_y, 300.0);
        assert_eq!(loaded_config.window.width, 500);
        assert_eq!(loaded_config.window.height, 600);
        assert_eq!(loaded_config.window.rotation, 45.0);
        assert_eq!(loaded_config.window.shape, "Heart");
        assert_eq!(loaded_config.camera.device_index, 1);

        // 清理测试文件
        let _ = fs::remove_file(&temp_config_path);
    }

    #[test]
    fn test_load_missing_config_creates_default() {
        // 创建临时配置文件路径
        let temp_dir = std::env::temp_dir();
        let temp_config_path = temp_dir.join("test_mira_missing_config.toml");
        
        // 确保测试开始时文件不存在
        let _ = fs::remove_file(&temp_config_path);

        // 创建配置管理器并设置临时路径
        let mut manager = ConfigManager::new().unwrap();
        manager.config_path = temp_config_path.clone();

        // 加载不存在的配置文件
        let load_result = manager.load();
        assert!(load_result.is_ok());
        
        // 应该创建默认配置文件
        assert!(temp_config_path.exists());
        
        let loaded_config = load_result.unwrap();
        assert_eq!(loaded_config.version, "1.0");
        assert_eq!(loaded_config.window.width, 400);
        assert_eq!(loaded_config.window.height, 400);

        // 清理测试文件
        let _ = fs::remove_file(&temp_config_path);
    }

    #[test]
    fn test_load_corrupted_config_creates_backup() {
        // 创建临时配置文件路径
        let temp_dir = std::env::temp_dir();
        let temp_config_path = temp_dir.join("test_mira_corrupted_config.toml");
        let backup_path = temp_config_path.with_extension("toml.backup");
        
        // 清理可能存在的文件
        let _ = fs::remove_file(&temp_config_path);
        let _ = fs::remove_file(&backup_path);

        // 创建损坏的配置文件
        fs::write(&temp_config_path, "invalid toml content [[[").unwrap();

        // 创建配置管理器并设置临时路径
        let mut manager = ConfigManager::new().unwrap();
        manager.config_path = temp_config_path.clone();

        // 加载损坏的配置文件
        let load_result = manager.load();
        assert!(load_result.is_ok());
        
        // 应该创建备份文件
        assert!(backup_path.exists());
        
        // 应该使用默认配置
        let loaded_config = load_result.unwrap();
        assert_eq!(loaded_config.version, "1.0");
        assert_eq!(loaded_config.window.width, 400);

        // 清理测试文件
        let _ = fs::remove_file(&temp_config_path);
        let _ = fs::remove_file(&backup_path);
    }

    #[test]
    fn test_config_migration() {
        let manager = ConfigManager::new().unwrap();
        
        // 测试从旧版本迁移
        let mut old_config = AppConfig {
            version: "0.9".to_string(),
            window: WindowConfig {
                position_x: 100.0,
                position_y: 100.0,
                width: 400,
                height: 400,
                rotation: 0.0,
                shape: "Circle".to_string(),
            },
            camera: CameraConfig { device_index: 0 },
        };

        // 检查是否需要迁移
        assert!(manager.needs_migration(&old_config));

        // 执行迁移
        let migrated = manager.migrate_config(old_config).unwrap();
        assert_eq!(migrated.version, "1.0");
    }

    #[test]
    fn test_enhanced_validation() {
        let manager = ConfigManager::new().unwrap();
        let mut config = AppConfig {
            version: "1.0".to_string(),
            window: WindowConfig {
                position_x: f64::NAN, // 无效位置
                position_y: f64::INFINITY, // 无效位置
                width: 50, // 小于最小值
                height: 5000, // 超过最大值
                rotation: f64::NAN, // 无效角度
                shape: "".to_string(), // 空形状名称
            },
            camera: CameraConfig { device_index: 999 }, // 过大的设备索引
        };

        manager.validate_and_fix_config(&mut config);

        // 验证修正结果
        assert_eq!(config.window.position_x, 100.0);
        assert_eq!(config.window.position_y, 100.0);
        assert_eq!(config.window.width, 100);
        assert_eq!(config.window.height, 4096);
        assert_eq!(config.window.rotation, 0.0);
        assert_eq!(config.window.shape, "Circle");
        assert_eq!(config.camera.device_index, 0);
    }

    #[test]
    fn test_window_size_limits() {
        let manager = ConfigManager::new().unwrap();
        let mut config = AppConfig {
            version: "1.0".to_string(),
            window: WindowConfig {
                position_x: 100.0,
                position_y: 100.0,
                width: 10000, // 超过最大值
                height: 10, // 小于最小值
                rotation: 0.0,
                shape: "Circle".to_string(),
            },
            camera: CameraConfig { device_index: 0 },
        };

        manager.validate_and_fix_config(&mut config);

        assert_eq!(config.window.width, 4096); // 修正为最大值
        assert_eq!(config.window.height, 100); // 修正为最小值
    }

    #[test]
    fn test_future_version_handling() {
        let manager = ConfigManager::new().unwrap();
        let future_config = AppConfig {
            version: "2.0".to_string(), // 未来版本
            window: WindowConfig {
                position_x: 100.0,
                position_y: 100.0,
                width: 400,
                height: 400,
                rotation: 0.0,
                shape: "Circle".to_string(),
            },
            camera: CameraConfig { device_index: 0 },
        };

        assert!(manager.needs_migration(&future_config));
        let migrated = manager.migrate_config(future_config).unwrap();
        assert_eq!(migrated.version, "1.0"); // 应该回退到默认配置
    }

    #[test]
    fn test_invalid_shape_names() {
        let manager = ConfigManager::new().unwrap();
        let invalid_shapes = ["", "Triangle", "Star", "InvalidShape"];
        
        for invalid_shape in &invalid_shapes {
            let mut config = AppConfig {
                version: "1.0".to_string(),
                window: WindowConfig {
                    position_x: 100.0,
                    position_y: 100.0,
                    width: 400,
                    height: 400,
                    rotation: 0.0,
                    shape: invalid_shape.to_string(),
                },
                camera: CameraConfig { device_index: 0 },
            };

            manager.validate_and_fix_config(&mut config);
            assert_eq!(config.window.shape, "Circle", "Invalid shape '{}' should be corrected to 'Circle'", invalid_shape);
        }
    }

    #[test]
    fn test_valid_shapes_preserved() {
        let manager = ConfigManager::new().unwrap();
        let valid_shapes = ["Circle", "Ellipse", "Rectangle", "RoundedRectangle", "Heart"];
        
        for valid_shape in &valid_shapes {
            let mut config = AppConfig {
                version: "1.0".to_string(),
                window: WindowConfig {
                    position_x: 100.0,
                    position_y: 100.0,
                    width: 400,
                    height: 400,
                    rotation: 0.0,
                    shape: valid_shape.to_string(),
                },
                camera: CameraConfig { device_index: 0 },
            };

            manager.validate_and_fix_config(&mut config);
            assert_eq!(config.window.shape, *valid_shape, "Valid shape '{}' should be preserved", valid_shape);
        }
    }
}
