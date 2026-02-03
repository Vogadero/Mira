// 上下文菜单实现
//
// 提供右键菜单功能，包括菜单项管理、布局计算、状态管理等

use log::{debug, error, info, warn};
use std::collections::HashMap;
use winit::dpi::PhysicalPosition;
use winit::dpi::PhysicalSize;

/// 菜单项回调函数类型
pub type MenuCallback = Box<dyn Fn() -> Result<(), String> + Send + Sync>;

/// 菜单项结构
#[derive(Debug)]
pub struct MenuItem {
    /// 菜单项ID（唯一标识符）
    pub id: String,
    /// 显示文本
    pub text: String,
    /// 图标名称（可选）
    pub icon: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 是否选中（用于单选/复选项）
    pub checked: bool,
    /// 菜单项类型
    pub item_type: MenuItemType,
    /// 分组ID（用于菜单项分组）
    pub group_id: Option<String>,
}

/// 菜单项类型
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemType {
    /// 普通菜单项
    Normal,
    /// 分隔线
    Separator,
    /// 单选项（互斥选择）
    Radio,
    /// 复选项
    Checkbox,
    /// 子菜单
    Submenu,
}

/// 菜单分组
#[derive(Debug)]
pub struct MenuGroup {
    /// 分组ID
    pub id: String,
    /// 分组标题
    pub title: String,
    /// 分组中的菜单项ID列表
    pub items: Vec<String>,
}

/// 菜单布局信息
#[derive(Debug, Clone)]
pub struct MenuLayout {
    /// 菜单位置
    pub position: PhysicalPosition<f32>,
    /// 菜单尺寸
    pub size: PhysicalSize<f32>,
    /// 菜单项高度
    pub item_height: f32,
    /// 菜单内边距
    pub padding: f32,
    /// 菜单边框宽度
    pub border_width: f32,
    /// 分隔线高度
    pub separator_height: f32,
    /// 最大宽度
    pub max_width: f32,
    /// 最小宽度
    pub min_width: f32,
}

impl Default for MenuLayout {
    fn default() -> Self {
        Self {
            position: PhysicalPosition::new(0.0, 0.0),
            size: PhysicalSize::new(200.0, 100.0),
            item_height: 24.0,
            padding: 8.0,
            border_width: 1.0,
            separator_height: 1.0,
            max_width: 300.0,
            min_width: 120.0,
        }
    }
}

/// 菜单状态
#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    /// 隐藏
    Hidden,
    /// 显示
    Visible,
    /// 动画中（显示）
    ShowingAnimation,
    /// 动画中（隐藏）
    HidingAnimation,
}

/// 上下文菜单管理器
pub struct ContextMenu {
    /// 菜单项列表
    items: HashMap<String, MenuItem>,
    /// 菜单分组
    groups: HashMap<String, MenuGroup>,
    /// 菜单项显示顺序
    display_order: Vec<String>,
    /// 当前菜单状态
    state: MenuState,
    /// 菜单布局
    layout: MenuLayout,
    /// 当前选中的菜单项ID
    selected_item: Option<String>,
    /// 当前悬浮的菜单项ID
    hovered_item: Option<String>,
    /// 菜单回调函数
    callbacks: HashMap<String, MenuCallback>,
    /// 屏幕尺寸（用于边界检查）
    screen_size: PhysicalSize<u32>,
}

impl ContextMenu {
    /// 创建新的上下文菜单
    pub fn new(screen_size: PhysicalSize<u32>) -> Self {
        info!("创建上下文菜单管理器");
        
        let mut menu = Self {
            items: HashMap::new(),
            groups: HashMap::new(),
            display_order: Vec::new(),
            state: MenuState::Hidden,
            layout: MenuLayout::default(),
            selected_item: None,
            hovered_item: None,
            callbacks: HashMap::new(),
            screen_size,
        };
        
        // 初始化默认菜单项
        menu.initialize_default_menu();
        
        menu
    }
    
    /// 初始化默认菜单项
    fn initialize_default_menu(&mut self) {
        debug!("初始化默认菜单项");
        
        // 形状选择分组
        self.add_group("shapes", "形状选择");
        self.add_menu_item(MenuItem {
            id: "shape_circle".to_string(),
            text: "圆形".to_string(),
            icon: Some("circle".to_string()),
            enabled: true,
            checked: true, // 默认选中
            item_type: MenuItemType::Radio,
            group_id: Some("shapes".to_string()),
        });
        self.add_menu_item(MenuItem {
            id: "shape_ellipse".to_string(),
            text: "椭圆形".to_string(),
            icon: Some("ellipse".to_string()),
            enabled: true,
            checked: false,
            item_type: MenuItemType::Radio,
            group_id: Some("shapes".to_string()),
        });
        self.add_menu_item(MenuItem {
            id: "shape_rectangle".to_string(),
            text: "矩形".to_string(),
            icon: Some("rectangle".to_string()),
            enabled: true,
            checked: false,
            item_type: MenuItemType::Radio,
            group_id: Some("shapes".to_string()),
        });
        self.add_menu_item(MenuItem {
            id: "shape_rounded_rectangle".to_string(),
            text: "圆角矩形".to_string(),
            icon: Some("rounded_rectangle".to_string()),
            enabled: true,
            checked: false,
            item_type: MenuItemType::Radio,
            group_id: Some("shapes".to_string()),
        });
        self.add_menu_item(MenuItem {
            id: "shape_heart".to_string(),
            text: "心形".to_string(),
            icon: Some("heart".to_string()),
            enabled: true,
            checked: false,
            item_type: MenuItemType::Radio,
            group_id: Some("shapes".to_string()),
        });
        
        // 分隔线
        self.add_menu_item(MenuItem {
            id: "separator_1".to_string(),
            text: "".to_string(),
            icon: None,
            enabled: false,
            checked: false,
            item_type: MenuItemType::Separator,
            group_id: None,
        });
        
        // 摄像头设备分组
        self.add_group("cameras", "摄像头设备");
        // 摄像头设备项将在运行时动态添加
        
        // 分隔线
        self.add_menu_item(MenuItem {
            id: "separator_2".to_string(),
            text: "".to_string(),
            icon: None,
            enabled: false,
            checked: false,
            item_type: MenuItemType::Separator,
            group_id: None,
        });
        
        // 窗口控制分组
        self.add_group("window_control", "窗口控制");
        self.add_menu_item(MenuItem {
            id: "reset_position".to_string(),
            text: "重置位置".to_string(),
            icon: Some("reset_position".to_string()),
            enabled: true,
            checked: false,
            item_type: MenuItemType::Normal,
            group_id: Some("window_control".to_string()),
        });
        self.add_menu_item(MenuItem {
            id: "reset_rotation".to_string(),
            text: "重置旋转".to_string(),
            icon: Some("reset_rotation".to_string()),
            enabled: true,
            checked: false,
            item_type: MenuItemType::Normal,
            group_id: Some("window_control".to_string()),
        });
        self.add_menu_item(MenuItem {
            id: "reset_size".to_string(),
            text: "重置大小".to_string(),
            icon: Some("reset_size".to_string()),
            enabled: true,
            checked: false,
            item_type: MenuItemType::Normal,
            group_id: Some("window_control".to_string()),
        });
        
        // 分隔线
        self.add_menu_item(MenuItem {
            id: "separator_3".to_string(),
            text: "".to_string(),
            icon: None,
            enabled: false,
            checked: false,
            item_type: MenuItemType::Separator,
            group_id: None,
        });
        
        // 状态信息分组
        self.add_group("status", "状态信息");
        self.add_menu_item(MenuItem {
            id: "show_info".to_string(),
            text: "显示信息".to_string(),
            icon: Some("info".to_string()),
            enabled: true,
            checked: false,
            item_type: MenuItemType::Normal,
            group_id: Some("status".to_string()),
        });
        
        debug!("默认菜单项初始化完成，共 {} 个菜单项", self.items.len());
    }
    
    /// 添加菜单分组
    pub fn add_group(&mut self, id: &str, title: &str) {
        debug!("添加菜单分组: {} - {}", id, title);
        
        let group = MenuGroup {
            id: id.to_string(),
            title: title.to_string(),
            items: Vec::new(),
        };
        
        self.groups.insert(id.to_string(), group);
    }
    
    /// 添加菜单项
    pub fn add_menu_item(&mut self, item: MenuItem) {
        debug!("添加菜单项: {} - {}", item.id, item.text);
        
        let item_id = item.id.clone();
        let group_id = item.group_id.clone();
        
        // 添加到菜单项列表
        self.items.insert(item_id.clone(), item);
        
        // 添加到显示顺序
        self.display_order.push(item_id.clone());
        
        // 如果有分组，添加到分组中
        if let Some(group_id) = group_id {
            if let Some(group) = self.groups.get_mut(&group_id) {
                group.items.push(item_id);
            } else {
                warn!("菜单项 {} 指定的分组 {} 不存在", item_id, group_id);
            }
        }
    }
    
    /// 移除菜单项
    pub fn remove_menu_item(&mut self, item_id: &str) {
        debug!("移除菜单项: {}", item_id);
        
        if let Some(item) = self.items.remove(item_id) {
            // 从显示顺序中移除
            self.display_order.retain(|id| id != item_id);
            
            // 从分组中移除
            if let Some(group_id) = &item.group_id {
                if let Some(group) = self.groups.get_mut(group_id) {
                    group.items.retain(|id| id != item_id);
                }
            }
            
            // 移除回调函数
            self.callbacks.remove(item_id);
            
            // 如果是当前选中或悬浮的项，清除状态
            if self.selected_item.as_ref() == Some(&item_id.to_string()) {
                self.selected_item = None;
            }
            if self.hovered_item.as_ref() == Some(&item_id.to_string()) {
                self.hovered_item = None;
            }
        } else {
            warn!("尝试移除不存在的菜单项: {}", item_id);
        }
    }
    
    /// 设置菜单项回调函数
    pub fn set_callback<F>(&mut self, item_id: &str, callback: F)
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        debug!("设置菜单项回调: {}", item_id);
        self.callbacks.insert(item_id.to_string(), Box::new(callback));
    }
    
    /// 显示菜单
    pub fn show(&mut self, position: PhysicalPosition<f32>) {
        info!("显示上下文菜单，位置: ({:.1}, {:.1})", position.x, position.y);
        
        // 计算菜单布局
        self.calculate_layout(position);
        
        // 设置状态为显示
        self.state = MenuState::Visible;
        
        debug!("菜单显示完成，尺寸: {:.1}x{:.1}", self.layout.size.width, self.layout.size.height);
    }
    
    /// 隐藏菜单
    pub fn hide(&mut self) {
        debug!("隐藏上下文菜单");
        
        self.state = MenuState::Hidden;
        self.selected_item = None;
        self.hovered_item = None;
    }
    
    /// 计算菜单布局
    fn calculate_layout(&mut self, requested_position: PhysicalPosition<f32>) {
        debug!("计算菜单布局，请求位置: ({:.1}, {:.1})", requested_position.x, requested_position.y);
        
        // 计算菜单内容尺寸
        let mut content_height = self.layout.padding * 2.0;
        let mut max_text_width = 0.0f32;
        
        for item_id in &self.display_order {
            if let Some(item) = self.items.get(item_id) {
                match item.item_type {
                    MenuItemType::Separator => {
                        content_height += self.layout.separator_height;
                    }
                    _ => {
                        content_height += self.layout.item_height;
                        // 估算文本宽度（简化计算，每个字符约8像素）
                        let text_width = item.text.chars().count() as f32 * 8.0;
                        max_text_width = max_text_width.max(text_width);
                    }
                }
            }
        }
        
        // 计算菜单宽度（文本宽度 + 图标空间 + 内边距）
        let icon_space = 24.0; // 图标空间
        let content_width = (max_text_width + icon_space + self.layout.padding * 2.0)
            .max(self.layout.min_width)
            .min(self.layout.max_width);
        
        // 设置菜单尺寸
        self.layout.size = PhysicalSize::new(content_width, content_height);
        
        // 边界检查和位置调整
        let adjusted_position = self.adjust_position_for_screen_bounds(requested_position);
        self.layout.position = adjusted_position;
        
        debug!("菜单布局计算完成: 位置({:.1}, {:.1}), 尺寸({:.1}x{:.1})", 
               self.layout.position.x, self.layout.position.y,
               self.layout.size.width, self.layout.size.height);
    }
    
    /// 调整菜单位置以适应屏幕边界
    fn adjust_position_for_screen_bounds(&self, requested_position: PhysicalPosition<f32>) -> PhysicalPosition<f32> {
        let screen_width = self.screen_size.width as f32;
        let screen_height = self.screen_size.height as f32;
        let menu_width = self.layout.size.width;
        let menu_height = self.layout.size.height;
        
        let mut adjusted_x = requested_position.x;
        let mut adjusted_y = requested_position.y;
        
        // 检查右边界
        if adjusted_x + menu_width > screen_width {
            adjusted_x = screen_width - menu_width;
            if adjusted_x < 0.0 {
                adjusted_x = 0.0;
            }
        }
        
        // 检查下边界
        if adjusted_y + menu_height > screen_height {
            adjusted_y = screen_height - menu_height;
            if adjusted_y < 0.0 {
                adjusted_y = 0.0;
            }
        }
        
        // 确保不超出左上边界
        adjusted_x = adjusted_x.max(0.0);
        adjusted_y = adjusted_y.max(0.0);
        
        if adjusted_x != requested_position.x || adjusted_y != requested_position.y {
            debug!("菜单位置已调整: ({:.1}, {:.1}) -> ({:.1}, {:.1})", 
                   requested_position.x, requested_position.y, adjusted_x, adjusted_y);
        }
        
        PhysicalPosition::new(adjusted_x, adjusted_y)
    }
    
    /// 检查点是否在菜单区域内
    pub fn is_point_inside(&self, position: PhysicalPosition<f32>) -> bool {
        if self.state != MenuState::Visible {
            return false;
        }
        
        let menu_left = self.layout.position.x;
        let menu_top = self.layout.position.y;
        let menu_right = menu_left + self.layout.size.width;
        let menu_bottom = menu_top + self.layout.size.height;
        
        position.x >= menu_left && position.x <= menu_right &&
        position.y >= menu_top && position.y <= menu_bottom
    }
    
    /// 获取指定位置的菜单项
    pub fn get_item_at_position(&self, position: PhysicalPosition<f32>) -> Option<&str> {
        if !self.is_point_inside(position) {
            return None;
        }
        
        let relative_y = position.y - self.layout.position.y - self.layout.padding;
        let mut current_y = 0.0;
        
        for item_id in &self.display_order {
            if let Some(item) = self.items.get(item_id) {
                let item_height = match item.item_type {
                    MenuItemType::Separator => self.layout.separator_height,
                    _ => self.layout.item_height,
                };
                
                if relative_y >= current_y && relative_y < current_y + item_height {
                    // 分隔线不可选择
                    if item.item_type != MenuItemType::Separator && item.enabled {
                        return Some(item_id);
                    }
                    break;
                }
                
                current_y += item_height;
            }
        }
        
        None
    }
    
    /// 设置悬浮菜单项
    pub fn set_hovered_item(&mut self, item_id: Option<String>) {
        if self.hovered_item != item_id {
            debug!("悬浮菜单项变更: {:?} -> {:?}", self.hovered_item, item_id);
            self.hovered_item = item_id;
        }
    }
    
    /// 执行菜单项
    pub fn execute_item(&mut self, item_id: &str) -> Result<(), String> {
        debug!("执行菜单项: {}", item_id);
        
        if let Some(item) = self.items.get(item_id) {
            if !item.enabled {
                warn!("尝试执行已禁用的菜单项: {}", item_id);
                return Err(format!("菜单项 {} 已禁用", item_id));
            }
            
            // 处理单选项逻辑
            if item.item_type == MenuItemType::Radio {
                self.handle_radio_selection(item_id);
            }
            
            // 执行回调函数
            if let Some(callback) = self.callbacks.get(item_id) {
                match callback() {
                    Ok(()) => {
                        info!("菜单项 {} 执行成功", item_id);
                        // 执行成功后隐藏菜单
                        self.hide();
                        Ok(())
                    }
                    Err(e) => {
                        error!("菜单项 {} 执行失败: {}", item_id, e);
                        Err(e)
                    }
                }
            } else {
                warn!("菜单项 {} 没有设置回调函数", item_id);
                self.hide();
                Ok(())
            }
        } else {
            error!("尝试执行不存在的菜单项: {}", item_id);
            Err(format!("菜单项 {} 不存在", item_id))
        }
    }
    
    /// 处理单选项选择逻辑
    fn handle_radio_selection(&mut self, selected_item_id: &str) {
        if let Some(selected_item) = self.items.get(selected_item_id) {
            if let Some(group_id) = &selected_item.group_id {
                // 取消同组其他项的选中状态
                for item_id in &self.display_order {
                    if let Some(item) = self.items.get_mut(item_id) {
                        if item.group_id.as_ref() == Some(group_id) && item.item_type == MenuItemType::Radio {
                            item.checked = item_id == selected_item_id;
                        }
                    }
                }
                debug!("单选组 {} 选择更新: {}", group_id, selected_item_id);
            }
        }
    }
    
    /// 更新摄像头设备列表
    pub fn update_camera_devices(&mut self, devices: &[(usize, String)], current_device: Option<usize>) {
        debug!("更新摄像头设备列表，设备数量: {}", devices.len());
        
        // 移除现有的摄像头菜单项
        let camera_items: Vec<String> = self.display_order.iter()
            .filter(|id| id.starts_with("camera_"))
            .cloned()
            .collect();
        
        for item_id in camera_items {
            self.remove_menu_item(&item_id);
        }
        
        // 找到摄像头分组的插入位置（在第一个分隔线之后）
        let insert_index = self.display_order.iter()
            .position(|id| id == "separator_1")
            .map(|pos| pos + 1)
            .unwrap_or(self.display_order.len());
        
        // 添加新的摄像头设备项
        for (device_index, device_name) in devices {
            let item_id = format!("camera_{}", device_index);
            let is_current = current_device == Some(*device_index);
            
            let item = MenuItem {
                id: item_id.clone(),
                text: device_name.clone(),
                icon: Some("camera".to_string()),
                enabled: true,
                checked: is_current,
                item_type: MenuItemType::Radio,
                group_id: Some("cameras".to_string()),
            };
            
            // 添加菜单项
            self.items.insert(item_id.clone(), item);
            
            // 插入到正确位置
            self.display_order.insert(insert_index + devices.iter().position(|(idx, _)| idx == device_index).unwrap(), item_id.clone());
            
            // 添加到分组
            if let Some(group) = self.groups.get_mut("cameras") {
                group.items.push(item_id);
            }
        }
        
        debug!("摄像头设备列表更新完成");
    }
    
    /// 更新状态信息
    pub fn update_status_info(&mut self, window_size: PhysicalSize<u32>, position: PhysicalPosition<f64>, rotation: f32) {
        // 更新显示信息菜单项的文本
        if let Some(item) = self.items.get_mut("show_info") {
            item.text = format!("尺寸: {}x{}, 位置: ({:.0}, {:.0}), 旋转: {:.0}°", 
                               window_size.width, window_size.height,
                               position.x, position.y,
                               rotation.to_degrees());
        }
    }
    
    /// 获取菜单状态
    pub fn state(&self) -> &MenuState {
        &self.state
    }
    
    /// 获取菜单布局
    pub fn layout(&self) -> &MenuLayout {
        &self.layout
    }
    
    /// 获取显示顺序的菜单项列表
    pub fn get_display_items(&self) -> Vec<&MenuItem> {
        self.display_order.iter()
            .filter_map(|id| self.items.get(id))
            .collect()
    }
    
    /// 获取当前悬浮的菜单项
    pub fn hovered_item(&self) -> Option<&str> {
        self.hovered_item.as_deref()
    }
    
    /// 更新屏幕尺寸
    pub fn update_screen_size(&mut self, screen_size: PhysicalSize<u32>) {
        if self.screen_size != screen_size {
            debug!("更新屏幕尺寸: {}x{} -> {}x{}", 
                   self.screen_size.width, self.screen_size.height,
                   screen_size.width, screen_size.height);
            self.screen_size = screen_size;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_creation() {
        let screen_size = PhysicalSize::new(1920, 1080);
        let menu = ContextMenu::new(screen_size);
        
        assert_eq!(menu.state, MenuState::Hidden);
        assert!(menu.items.len() > 0);
        assert!(menu.groups.len() > 0);
    }

    #[test]
    fn test_menu_item_management() {
        let screen_size = PhysicalSize::new(1920, 1080);
        let mut menu = ContextMenu::new(screen_size);
        
        let initial_count = menu.items.len();
        
        // 添加测试菜单项
        let test_item = MenuItem {
            id: "test_item".to_string(),
            text: "测试项".to_string(),
            icon: None,
            enabled: true,
            checked: false,
            item_type: MenuItemType::Normal,
            group_id: None,
        };
        
        menu.add_menu_item(test_item);
        assert_eq!(menu.items.len(), initial_count + 1);
        assert!(menu.items.contains_key("test_item"));
        
        // 移除测试菜单项
        menu.remove_menu_item("test_item");
        assert_eq!(menu.items.len(), initial_count);
        assert!(!menu.items.contains_key("test_item"));
    }

    #[test]
    fn test_menu_layout_calculation() {
        let screen_size = PhysicalSize::new(1920, 1080);
        let mut menu = ContextMenu::new(screen_size);
        
        let position = PhysicalPosition::new(100.0, 100.0);
        menu.show(position);
        
        assert_eq!(menu.state, MenuState::Visible);
        assert!(menu.layout.size.width > 0.0);
        assert!(menu.layout.size.height > 0.0);
    }

    #[test]
    fn test_boundary_adjustment() {
        let screen_size = PhysicalSize::new(800, 600);
        let mut menu = ContextMenu::new(screen_size);
        
        // 测试右边界调整
        let position = PhysicalPosition::new(750.0, 100.0);
        menu.show(position);
        
        // 菜单应该被调整到屏幕内
        assert!(menu.layout.position.x + menu.layout.size.width <= screen_size.width as f32);
        
        // 测试下边界调整
        let position = PhysicalPosition::new(100.0, 550.0);
        menu.show(position);
        
        // 菜单应该被调整到屏幕内
        assert!(menu.layout.position.y + menu.layout.size.height <= screen_size.height as f32);
    }

    #[test]
    fn test_point_inside_detection() {
        let screen_size = PhysicalSize::new(1920, 1080);
        let mut menu = ContextMenu::new(screen_size);
        
        let position = PhysicalPosition::new(100.0, 100.0);
        menu.show(position);
        
        // 测试菜单内的点
        let inside_point = PhysicalPosition::new(
            menu.layout.position.x + menu.layout.size.width / 2.0,
            menu.layout.position.y + menu.layout.size.height / 2.0,
        );
        assert!(menu.is_point_inside(inside_point));
        
        // 测试菜单外的点
        let outside_point = PhysicalPosition::new(0.0, 0.0);
        assert!(!menu.is_point_inside(outside_point));
    }

    #[test]
    fn test_radio_selection() {
        let screen_size = PhysicalSize::new(1920, 1080);
        let mut menu = ContextMenu::new(screen_size);
        
        // 测试形状选择（单选组）
        menu.execute_item("shape_ellipse").ok();
        
        // 检查选中状态
        assert!(menu.items.get("shape_ellipse").unwrap().checked);
        assert!(!menu.items.get("shape_circle").unwrap().checked);
    }

    #[test]
    fn test_camera_device_update() {
        let screen_size = PhysicalSize::new(1920, 1080);
        let mut menu = ContextMenu::new(screen_size);
        
        let devices = vec![
            (0, "内置摄像头".to_string()),
            (1, "USB摄像头".to_string()),
        ];
        
        menu.update_camera_devices(&devices, Some(0));
        
        // 检查摄像头菜单项是否添加
        assert!(menu.items.contains_key("camera_0"));
        assert!(menu.items.contains_key("camera_1"));
        
        // 检查当前设备选中状态
        assert!(menu.items.get("camera_0").unwrap().checked);
        assert!(!menu.items.get("camera_1").unwrap().checked);
    }
}