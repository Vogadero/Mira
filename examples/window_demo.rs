// 窗口功能演示程序

use mira::window::WindowManager;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use log::{info, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    env_logger::init();
    
    info!("窗口演示程序启动");
    
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
    
    info!("窗口初始状态:");
    info!("  位置: {:?}", window_manager.position());
    info!("  尺寸: {:?}", window_manager.size());
    info!("  旋转角度: {:.1}°", window_manager.rotation());
    
    // 存储当前鼠标位置
    let mut current_cursor_pos = PhysicalPosition::new(0.0, 0.0);
    
    // 运行事件循环
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("收到窗口关闭请求");
                *control_flow = ControlFlow::Exit;
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
                use winit::event::{ElementState, MouseButton};
                
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
                
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        if y > 0.0 {
                            // 向上滚动，放大 10%
                            window_manager.scale(1.1);
                            info!("窗口放大，新尺寸: {:?}", window_manager.size());
                        } else if y < 0.0 {
                            // 向下滚动，缩小 10%
                            window_manager.scale(0.9);
                            info!("窗口缩小，新尺寸: {:?}", window_manager.size());
                        }
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                // 这里将来会调用渲染引擎
                // 目前只是一个占位符
            }
            _ => {}
        }
    })?;
    
    Ok(())
}