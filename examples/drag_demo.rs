// 窗口拖拽功能演示程序
// 演示完整的拖拽功能，包括性能测试

use mira::window::WindowManager;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};
use log::{info, error, warn};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    env_logger::init();
    
    info!("窗口拖拽演示程序启动");
    
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
    
    info!("拖拽功能演示:");
    info!("  - 按住鼠标左键拖拽窗口");
    info!("  - 窗口会跟随鼠标移动");
    info!("  - 窗口会被约束在屏幕边界内（至少20%可见）");
    info!("  - 拖拽响应时间 < 16ms");
    
    // 存储当前鼠标位置和性能统计
    let mut current_cursor_pos = PhysicalPosition::new(0.0, 0.0);
    let mut drag_start_time: Option<Instant> = None;
    let mut drag_update_count = 0;
    let mut total_drag_time = std::time::Duration::new(0, 0);
    
    // 运行事件循环
    event_loop.run(move |event, event_loop| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("收到窗口关闭请求");
                
                // 输出性能统计
                if drag_update_count > 0 {
                    let avg_time = total_drag_time.as_micros() / drag_update_count as u128;
                    info!("拖拽性能统计:");
                    info!("  总更新次数: {}", drag_update_count);
                    info!("  平均响应时间: {:.2}ms", avg_time as f64 / 1000.0);
                    
                    if avg_time < 16000 {
                        info!("  ✓ 拖拽响应时间符合要求 (< 16ms)");
                    } else {
                        warn!("  ✗ 拖拽响应时间超出要求 (>= 16ms)");
                    }
                }
                
                event_loop.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                // 更新当前鼠标位置
                current_cursor_pos = position;
                
                // 如果正在拖拽，更新窗口位置并测量性能
                if window_manager.is_dragging() {
                    let update_start = Instant::now();
                    
                    window_manager.update_drag(position);
                    
                    let update_time = update_start.elapsed();
                    total_drag_time += update_time;
                    drag_update_count += 1;
                    
                    // 如果单次更新时间超过 16ms，发出警告
                    if update_time.as_millis() >= 16 {
                        warn!("拖拽更新时间过长: {:.2}ms", update_time.as_millis());
                    }
                    
                    // 每 100 次更新输出一次统计
                    if drag_update_count % 100 == 0 {
                        let avg_time = total_drag_time.as_micros() / drag_update_count as u128;
                        info!("拖拽性能 ({}次更新): 平均 {:.2}ms", 
                              drag_update_count, avg_time as f64 / 1000.0);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                use winit::event::{ElementState, MouseButton};
                
                match (state, button) {
                    (ElementState::Pressed, MouseButton::Left) => {
                        // 开始拖拽
                        window_manager.start_drag(current_cursor_pos);
                        drag_start_time = Some(Instant::now());
                        
                        info!("开始拖拽");
                        info!("  鼠标位置: ({:.1}, {:.1})", current_cursor_pos.x, current_cursor_pos.y);
                        info!("  窗口位置: {:?}", window_manager.position());
                    }
                    (ElementState::Released, MouseButton::Left) => {
                        if window_manager.is_dragging() {
                            window_manager.end_drag();
                            
                            if let Some(start_time) = drag_start_time {
                                let drag_duration = start_time.elapsed();
                                info!("结束拖拽");
                                info!("  拖拽持续时间: {:.2}s", drag_duration.as_secs_f64());
                                info!("  最终窗口位置: {:?}", window_manager.position());
                                info!("  本次拖拽更新次数: {}", drag_update_count);
                            }
                            
                            drag_start_time = None;
                        }
                    }
                    _ => {}
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                use winit::event::{ElementState, VirtualKeyCode};
                
                if let Some(keycode) = input.virtual_keycode {
                    if input.state == ElementState::Pressed {
                        match keycode {
                            VirtualKeyCode::R => {
                                // 重置窗口位置
                                window_manager.set_position(100.0, 100.0);
                                info!("窗口位置已重置到 (100, 100)");
                            }
                            VirtualKeyCode::T => {
                                // 测试边界约束
                                info!("测试边界约束...");
                                window_manager.set_position(-500.0, -500.0);
                                window_manager.constrain_to_screen(winit::dpi::PhysicalSize::new(1920, 1080));
                                info!("约束后位置: {:?}", window_manager.position());
                            }
                            VirtualKeyCode::Escape => {
                                info!("用户按下 ESC 键，退出程序");
                                event_loop.exit();
                            }
                            _ => {}
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