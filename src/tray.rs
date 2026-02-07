// 系统托盘图标管理
//
// 提供系统托盘图标和右键菜单功能

use log::{debug, info, warn};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
    Icon, TrayIcon, TrayIconBuilder,
};

/// 托盘图标管理器
pub struct TrayManager {
    _tray_icon: TrayIcon,
    menu: Menu,
    
    // 菜单项
    shape_circle: MenuItem,
    shape_ellipse: MenuItem,
    shape_rectangle: MenuItem,
    shape_rounded_rectangle: MenuItem,
    shape_heart: MenuItem,
    
    reset_position: MenuItem,
    reset_rotation: MenuItem,
    reset_size: MenuItem,
    
    rotate_left: MenuItem,
    rotate_right: MenuItem,
    
    show_info: MenuItem,
    quit: MenuItem,
}

impl TrayManager {
    /// 创建默认图标（一个简单的圆形摄像头图标）
    fn create_default_icon() -> Result<Icon, String> {
        // 创建一个 32x32 的 RGBA 图标
        let size = 32;
        let mut rgba = vec![0u8; (size * size * 4) as usize];
        
        let center = size as f32 / 2.0;
        let radius = 12.0;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let distance = (dx * dx + dy * dy).sqrt();
                
                let idx = ((y * size + x) * 4) as usize;
                
                if distance <= radius {
                    // 内圆 - 蓝色（代表摄像头镜头）
                    rgba[idx] = 64;      // R
                    rgba[idx + 1] = 128; // G
                    rgba[idx + 2] = 255; // B
                    rgba[idx + 3] = 255; // A
                } else if distance <= radius + 2.0 {
                    // 外圈 - 白色边框
                    rgba[idx] = 255;     // R
                    rgba[idx + 1] = 255; // G
                    rgba[idx + 2] = 255; // B
                    rgba[idx + 3] = 255; // A
                } else {
                    // 透明背景
                    rgba[idx + 3] = 0;
                }
            }
        }
        
        Icon::from_rgba(rgba, size, size)
            .map_err(|e| format!("创建图标失败: {}", e))
    }
    
    /// 创建托盘管理器
    pub fn new() -> Result<Self, String> {
        info!("创建系统托盘图标");
        
        // 创建菜单
        let menu = Menu::new();
        
        // 形状选择子菜单
        let shape_menu = Submenu::new("形状选择", true);
        let shape_circle = MenuItem::new("圆形 (F1)", true, None);
        let shape_ellipse = MenuItem::new("椭圆形 (F2)", true, None);
        let shape_rectangle = MenuItem::new("矩形 (F3)", true, None);
        let shape_rounded_rectangle = MenuItem::new("圆角矩形 (F4)", true, None);
        let shape_heart = MenuItem::new("心形 (F5)", true, None);
        
        shape_menu.append(&shape_circle).map_err(|e| format!("添加菜单项失败: {}", e))?;
        shape_menu.append(&shape_ellipse).map_err(|e| format!("添加菜单项失败: {}", e))?;
        shape_menu.append(&shape_rectangle).map_err(|e| format!("添加菜单项失败: {}", e))?;
        shape_menu.append(&shape_rounded_rectangle).map_err(|e| format!("添加菜单项失败: {}", e))?;
        shape_menu.append(&shape_heart).map_err(|e| format!("添加菜单项失败: {}", e))?;
        
        menu.append(&shape_menu).map_err(|e| format!("添加子菜单失败: {}", e))?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| format!("添加分隔符失败: {}", e))?;
        
        // 窗口控制子菜单
        let window_menu = Submenu::new("窗口控制", true);
        let reset_position = MenuItem::new("重置位置", true, None);
        let reset_rotation = MenuItem::new("重置旋转", true, None);
        let reset_size = MenuItem::new("重置大小", true, None);
        
        window_menu.append(&reset_position).map_err(|e| format!("添加菜单项失败: {}", e))?;
        window_menu.append(&reset_rotation).map_err(|e| format!("添加菜单项失败: {}", e))?;
        window_menu.append(&reset_size).map_err(|e| format!("添加菜单项失败: {}", e))?;
        
        menu.append(&window_menu).map_err(|e| format!("添加子菜单失败: {}", e))?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| format!("添加分隔符失败: {}", e))?;
        
        // 旋转控制子菜单
        let rotate_menu = Submenu::new("旋转控制", true);
        let rotate_left = MenuItem::new("逆时针旋转 15° (Ctrl+滚轮下)", true, None);
        let rotate_right = MenuItem::new("顺时针旋转 15° (Ctrl+滚轮上)", true, None);
        
        rotate_menu.append(&rotate_left).map_err(|e| format!("添加菜单项失败: {}", e))?;
        rotate_menu.append(&rotate_right).map_err(|e| format!("添加菜单项失败: {}", e))?;
        
        menu.append(&rotate_menu).map_err(|e| format!("添加子菜单失败: {}", e))?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| format!("添加分隔符失败: {}", e))?;
        
        // 其他功能
        let show_info = MenuItem::new("显示信息", true, None);
        menu.append(&show_info).map_err(|e| format!("添加菜单项失败: {}", e))?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| format!("添加分隔符失败: {}", e))?;
        
        let quit = MenuItem::new("退出", true, None);
        menu.append(&quit).map_err(|e| format!("添加菜单项失败: {}", e))?;
        
        // 创建一个简单的图标（32x32 RGBA）
        let icon = Self::create_default_icon()?;
        
        // 创建托盘图标
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("Mira - 桌面摄像精灵\n右键点击显示菜单")
            .with_icon(icon)
            .build()
            .map_err(|e| format!("创建托盘图标失败: {}", e))?;
        
        info!("系统托盘图标创建成功");
        
        Ok(Self {
            _tray_icon: tray_icon,
            menu,
            shape_circle,
            shape_ellipse,
            shape_rectangle,
            shape_rounded_rectangle,
            shape_heart,
            reset_position,
            reset_rotation,
            reset_size,
            rotate_left,
            rotate_right,
            show_info,
            quit,
        })
    }
    
    /// 处理菜单事件
    pub fn handle_menu_event(&self) -> Option<TrayMenuAction> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            debug!("收到托盘菜单事件: {:?}", event.id);
            
            if event.id == self.shape_circle.id() {
                return Some(TrayMenuAction::ShapeCircle);
            } else if event.id == self.shape_ellipse.id() {
                return Some(TrayMenuAction::ShapeEllipse);
            } else if event.id == self.shape_rectangle.id() {
                return Some(TrayMenuAction::ShapeRectangle);
            } else if event.id == self.shape_rounded_rectangle.id() {
                return Some(TrayMenuAction::ShapeRoundedRectangle);
            } else if event.id == self.shape_heart.id() {
                return Some(TrayMenuAction::ShapeHeart);
            } else if event.id == self.reset_position.id() {
                return Some(TrayMenuAction::ResetPosition);
            } else if event.id == self.reset_rotation.id() {
                return Some(TrayMenuAction::ResetRotation);
            } else if event.id == self.reset_size.id() {
                return Some(TrayMenuAction::ResetSize);
            } else if event.id == self.rotate_left.id() {
                return Some(TrayMenuAction::RotateLeft);
            } else if event.id == self.rotate_right.id() {
                return Some(TrayMenuAction::RotateRight);
            } else if event.id == self.show_info.id() {
                return Some(TrayMenuAction::ShowInfo);
            } else if event.id == self.quit.id() {
                return Some(TrayMenuAction::Quit);
            }
        }
        
        None
    }
}

/// 托盘菜单动作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayMenuAction {
    ShapeCircle,
    ShapeEllipse,
    ShapeRectangle,
    ShapeRoundedRectangle,
    ShapeHeart,
    ResetPosition,
    ResetRotation,
    ResetSize,
    RotateLeft,
    RotateRight,
    ShowInfo,
    Quit,
}
