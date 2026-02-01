// Mira - 桌面摄像精灵库

pub mod camera;
pub mod config;
pub mod error;
pub mod event;
pub mod logging;
pub mod memory;
pub mod performance;
pub mod render;
pub mod shape;
pub mod window;

// 重新导出常用类型
pub use error::{CameraError, ConfigError, MiraError, RenderError, WindowError};
pub use event::EventHandler;