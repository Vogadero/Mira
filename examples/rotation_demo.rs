// 窗口旋转功能演示

use log::{error, info};
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent, ElementState, MouseButton},
    keyboard::ModifiersState,
    event_loop::EventLoop,
};
use mira::window::WindowManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    env_logger::init();
    
    info!("窗口旋转功能演示启动中...");
    
    // 创建事件循环
    let event_loop = EventLoop::new()?;
    
    // 创建窗口管理器
    let mut window_manager = match WindowManager::new(&event_loop) {
        Ok(manager) => {
            info!("窗口管理器创建成功");
            manager
        }
        Err(e) => {
            error!("窗口管理器创建失败: {}", e);
            return Err(e.into());
        }
    };
    
    info!("窗口旋转功能演示");
    info!("功能特性:");
    info!("  - 旋转角度存储和管理");
    info!("  - Ctrl + 鼠标滚轮旋转 (每次 ±15°)");
    info!("  - 角度归一化 (0-360° 范围)");
    info!("  - 自动对齐逻辑 (0°、90°、180°、270° ±5° 范围内自动对齐)");
    
    // 存储当前鼠标位置和修饰键状态
    let mut current_cursor_pos = PhysicalPosition::new(0.0, 0.0);
    let mut modifiers_state = ModifiersState::empty();
    
    info!("启动事件循环...");
    info!("操作说明:");
    info!("  - 左键拖拽: 移动窗口");
    info!("  - 鼠标滚轮: 缩放窗口");
    info!("  - Ctrl + 鼠标滚轮向上: 顺时针旋转 (+15°)");
    info!("  - Ctrl + 鼠标滚轮向下: 逆时针旋转 (-15°)");
    info!("  - 自动对齐: 在 0°、90°、180°、270° ±5° 范围内自动对齐");
    info!("  - 关闭窗口: 退出程序");
    
    // 运行事件循环
    event_loop.run(move |event, event_loop| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("收到窗口关闭请求");
                event_loop.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(new_modifiers),
                ..
            } => {
                // 更新修饰键状态
                modifiers_state = new_modifiers.state();
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                // 更新当前鼠标位置
                current_cursor_pos = position;
                
                // 如果正在拖拽，更新窗口位置
                if window_manager.is_dragging() {
                    window_manager.update_drag(position);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                match (state, button) {
                    (ElementState::Pressed, MouseButton::Left) => {
                        // 开始拖拽，使用当前鼠标位置
                        window_manager.start_drag(current_cursor_pos);
                        info!("开始拖拽，鼠标位置: {:?}", current_cursor_pos);
                    }
                    (ElementState::Released, MouseButton::Left) => {
                        window_manager.end_drag();
                        info!("结束拖拽，最终位置: {:?}", window_manager.position());
                    }
                    _ => {}
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                use winit::event::MouseScrollDelta;
                
                // 检查是否按下了 Ctrl 键
                let ctrl_pressed = modifiers_state.control_key();
                
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        if ctrl_pressed {
                            // Ctrl + 滚轮：旋转窗口
                            let old_rotation = window_manager.rotation();
                            
                            if y > 0.0 {
                                // 向上滚动，顺时针旋转 15 度
                                window_manager.rotate(15.0);
                                info!("窗口顺时针旋转 15°，角度: {:.1}° -> {:.1}°", old_rotation, window_manager.rotation());
                            } else if y < 0.0 {
                                // 向下滚动，逆时针旋转 15 度
                                window_manager.rotate(-15.0);
                                info!("窗口逆时针旋转 15°，角度: {:.1}° -> {:.1}°", old_rotation, window_manager.rotation());
                            }
                        } else {
                            // 普通滚轮：缩放窗口
                            let old_size = window_manager.size();
                            
                            if y > 0.0 {
                                // 向上滚动，放大 10%
                                window_manager.scale(1.1);
                                info!("窗口放大 10%，尺寸: {:?} -> {:?}", old_size, window_manager.size());
                            } else if y < 0.0 {
                                // 向下滚动，缩小 10% (1/1.1 ≈ 0.909)
                                window_manager.scale(1.0 / 1.1);
                                info!("窗口缩小 10%，尺寸: {:?} -> {:?}", old_size, window_manager.size());
                            }
                        }
                    }
                    MouseScrollDelta::PixelDelta(delta) => {
                        // 处理像素级滚动（触摸板等）
                        let y = delta.y as f32;
                        if y.abs() > 1.0 {
                            if ctrl_pressed {
                                // Ctrl + 触摸板滚动：旋转窗口
                                let old_rotation = window_manager.rotation();
                                
                                if y > 0.0 {
                                    // 向上滚动，顺时针旋转 15 度
                                    window_manager.rotate(15.0);
                                    info!("窗口顺时针旋转 15°，角度: {:.1}° -> {:.1}°", old_rotation, window_manager.rotation());
                                } else {
                                    // 向下滚动，逆时针旋转 15 度
                                    window_manager.rotate(-15.0);
                                    info!("窗口逆时针旋转 15°，角度: {:.1}° -> {:.1}°", old_rotation, window_manager.rotation());
                                }
                            } else {
                                // 普通触摸板滚动：缩放窗口
                                let old_size = window_manager.size();
                                
                                if y > 0.0 {
                                    // 向上滚动，放大 10%
                                    window_manager.scale(1.1);
                                    info!("窗口放大 10%，尺寸: {:?} -> {:?}", old_size, window_manager.size());
                                } else {
                                    // 向下滚动，缩小 10%
                                    window_manager.scale(1.0 / 1.1);
                                    info!("窗口缩小 10%，尺寸: {:?} -> {:?}", old_size, window_manager.size());
                                }
                            }
                        }
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // 这里将来会调用渲染引擎
                // 目前只是一个占位符
            }
            _ => {}
        }
    })?;
    
    Ok(())
}