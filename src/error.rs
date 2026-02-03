// 错误类型定义

use std::fmt;

/// Mira 应用程序的顶层错误类型
#[derive(Debug)]
pub enum MiraError {
    /// 摄像头相关错误
    Camera(CameraError),
    /// 窗口相关错误
    Window(WindowError),
    /// 渲染相关错误
    Render(RenderError),
    /// 配置相关错误
    Config(ConfigError),
}

/// 摄像头错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum CameraError {
    /// 未找到摄像头设备
    NoDeviceFound,
    /// 设备正被其他应用使用
    DeviceInUse,
    /// 摄像头访问权限被拒绝
    PermissionDenied,
    /// 视频捕获错误
    CaptureError(String),
}

/// 窗口错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum WindowError {
    /// 窗口创建失败
    CreationFailed(String),
    /// 无效的窗口尺寸
    InvalidSize,
    /// 无效的窗口位置
    InvalidPosition,
}

/// 渲染错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum RenderError {
    /// GPU 初始化失败
    InitializationFailed(String),
    /// 纹理上传失败
    TextureUploadFailed,
    /// 渲染失败
    RenderFailed(String),
    /// UI 渲染失败
    UIRenderFailed(String),
}

/// 配置错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// 配置文件未找到
    FileNotFound,
    /// 配置解析错误
    ParseError(String),
    /// 配置写入错误
    WriteError(String),
}

// 实现 Display trait 用于友好的错误消息

impl fmt::Display for MiraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MiraError::Camera(e) => write!(f, "摄像头错误: {}", e),
            MiraError::Window(e) => write!(f, "窗口错误: {}", e),
            MiraError::Render(e) => write!(f, "渲染错误: {}", e),
            MiraError::Config(e) => write!(f, "配置错误: {}", e),
        }
    }
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CameraError::NoDeviceFound => {
                write!(f, "未检测到摄像头设备，请连接摄像头后重试")
            }
            CameraError::DeviceInUse => {
                write!(f, "摄像头正被其他应用使用，请关闭占用摄像头的应用")
            }
            CameraError::PermissionDenied => {
                write!(f, "摄像头访问权限被拒绝，请在系统设置中允许 Mira 访问摄像头")
            }
            CameraError::CaptureError(msg) => write!(f, "视频捕获失败: {}", msg),
        }
    }
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WindowError::CreationFailed(msg) => write!(f, "窗口创建失败: {}", msg),
            WindowError::InvalidSize => write!(f, "无效的窗口尺寸"),
            WindowError::InvalidPosition => write!(f, "无效的窗口位置"),
        }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RenderError::InitializationFailed(msg) => {
                write!(f, "GPU 初始化失败，请更新显卡驱动: {}", msg)
            }
            RenderError::TextureUploadFailed => write!(f, "纹理上传失败"),
            RenderError::RenderFailed(msg) => write!(f, "渲染失败: {}", msg),
            RenderError::UIRenderFailed(msg) => write!(f, "UI 渲染失败: {}", msg),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::FileNotFound => write!(f, "配置文件未找到"),
            ConfigError::ParseError(msg) => write!(f, "配置解析错误: {}", msg),
            ConfigError::WriteError(msg) => write!(f, "配置写入错误: {}", msg),
        }
    }
}

// 实现 std::error::Error trait

impl std::error::Error for MiraError {}
impl std::error::Error for CameraError {}
impl std::error::Error for WindowError {}
impl std::error::Error for RenderError {}
impl std::error::Error for ConfigError {}

// 实现 From trait 用于错误转换

impl From<CameraError> for MiraError {
    fn from(err: CameraError) -> Self {
        MiraError::Camera(err)
    }
}

impl From<WindowError> for MiraError {
    fn from(err: WindowError) -> Self {
        MiraError::Window(err)
    }
}

impl From<RenderError> for MiraError {
    fn from(err: RenderError) -> Self {
        MiraError::Render(err)
    }
}

impl From<ConfigError> for MiraError {
    fn from(err: ConfigError) -> Self {
        MiraError::Config(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_error_display() {
        let err = CameraError::NoDeviceFound;
        assert!(err.to_string().contains("未检测到摄像头设备"));

        let err = CameraError::DeviceInUse;
        assert!(err.to_string().contains("摄像头正被其他应用使用"));

        let err = CameraError::PermissionDenied;
        assert!(err.to_string().contains("摄像头访问权限被拒绝"));

        let err = CameraError::CaptureError("test error".to_string());
        assert!(err.to_string().contains("视频捕获失败"));
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_window_error_display() {
        let err = WindowError::CreationFailed("test".to_string());
        assert!(err.to_string().contains("窗口创建失败"));

        let err = WindowError::InvalidSize;
        assert!(err.to_string().contains("无效的窗口尺寸"));

        let err = WindowError::InvalidPosition;
        assert!(err.to_string().contains("无效的窗口位置"));
    }

    #[test]
    fn test_render_error_display() {
        let err = RenderError::InitializationFailed("test".to_string());
        assert!(err.to_string().contains("GPU 初始化失败"));

        let err = RenderError::TextureUploadFailed;
        assert!(err.to_string().contains("纹理上传失败"));

        let err = RenderError::RenderFailed("test".to_string());
        assert!(err.to_string().contains("渲染失败"));

        let err = RenderError::UIRenderFailed("test".to_string());
        assert!(err.to_string().contains("UI 渲染失败"));
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::FileNotFound;
        assert!(err.to_string().contains("配置文件未找到"));

        let err = ConfigError::ParseError("test".to_string());
        assert!(err.to_string().contains("配置解析错误"));

        let err = ConfigError::WriteError("test".to_string());
        assert!(err.to_string().contains("配置写入错误"));
    }

    #[test]
    fn test_mira_error_from_camera_error() {
        let camera_err = CameraError::NoDeviceFound;
        let mira_err: MiraError = camera_err.into();
        assert!(matches!(mira_err, MiraError::Camera(_)));
    }

    #[test]
    fn test_mira_error_from_window_error() {
        let window_err = WindowError::InvalidSize;
        let mira_err: MiraError = window_err.into();
        assert!(matches!(mira_err, MiraError::Window(_)));
    }

    #[test]
    fn test_mira_error_from_render_error() {
        let render_err = RenderError::TextureUploadFailed;
        let mira_err: MiraError = render_err.into();
        assert!(matches!(mira_err, MiraError::Render(_)));
    }

    #[test]
    fn test_mira_error_from_config_error() {
        let config_err = ConfigError::FileNotFound;
        let mira_err: MiraError = config_err.into();
        assert!(matches!(mira_err, MiraError::Config(_)));
    }

    #[test]
    fn test_mira_error_display() {
        let err = MiraError::Camera(CameraError::NoDeviceFound);
        assert!(err.to_string().contains("摄像头错误"));

        let err = MiraError::Window(WindowError::InvalidSize);
        assert!(err.to_string().contains("窗口错误"));

        let err = MiraError::Render(RenderError::TextureUploadFailed);
        assert!(err.to_string().contains("渲染错误"));

        let err = MiraError::Config(ConfigError::FileNotFound);
        assert!(err.to_string().contains("配置错误"));
    }
}
