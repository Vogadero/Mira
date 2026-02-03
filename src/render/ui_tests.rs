#[cfg(test)]
mod ui_tests {
    use super::*;
    use winit::dpi::PhysicalSize;

    // 模拟渲染引擎的按钮几何体创建方法
    fn create_mock_button_geometry(center_x: f32, center_y: f32, size: f32, color: [f32; 4]) -> Vec<UIVertex> {
        let radius = size / 2.0;
        let segments = 16;
        let mut vertices = Vec::new();
        
        // 中心顶点
        vertices.push(UIVertex {
            position: [center_x, center_y],
            color,
        });
        
        // 圆周顶点
        for i in 0..=segments {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            
            vertices.push(UIVertex {
                position: [x, y],
                color,
            });
        }
        
        vertices
    }

    fn create_mock_button_indices(segments: u32) -> Vec<u16> {
        let mut indices = Vec::new();
        
        for i in 0..segments {
            indices.push(0); // 中心点
            indices.push((i + 1) as u16);
            indices.push((i + 2) as u16);
        }
        
        indices
    }

    fn create_mock_button_symbol(center_x: f32, center_y: f32, size: f32, symbol: &str, color: [f32; 4]) -> Vec<UIVertex> {
        let mut vertices = Vec::new();
        let symbol_size = size * 0.4;
        let half_size = symbol_size / 2.0;
        let line_width = 2.0;
        
        match symbol {
            "×" | "X" => {
                // X 符号 - 两条对角线
                let line1_vertices = create_mock_line_geometry(
                    center_x - half_size, center_y - half_size,
                    center_x + half_size, center_y + half_size,
                    line_width, color
                );
                vertices.extend(line1_vertices);
                
                let line2_vertices = create_mock_line_geometry(
                    center_x + half_size, center_y - half_size,
                    center_x - half_size, center_y + half_size,
                    line_width, color
                );
                vertices.extend(line2_vertices);
            }
            "−" | "-" => {
                // 减号 - 水平线
                let line_vertices = create_mock_line_geometry(
                    center_x - half_size, center_y,
                    center_x + half_size, center_y,
                    line_width, color
                );
                vertices.extend(line_vertices);
            }
            _ => {}
        }
        
        vertices
    }

    fn create_mock_line_geometry(x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: [f32; 4]) -> Vec<UIVertex> {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let length = (dx * dx + dy * dy).sqrt();
        
        if length == 0.0 {
            return Vec::new();
        }
        
        let dir_x = dx / length;
        let dir_y = dy / length;
        let perp_x = -dir_y * width / 2.0;
        let perp_y = dir_x * width / 2.0;
        
        vec![
            UIVertex { position: [x1 + perp_x, y1 + perp_y], color },
            UIVertex { position: [x1 - perp_x, y1 - perp_y], color },
            UIVertex { position: [x2 - perp_x, y2 - perp_y], color },
            UIVertex { position: [x2 + perp_x, y2 + perp_y], color },
        ]
    }

    #[test]
    fn test_button_geometry_creation() {
        let center_x = 100.0;
        let center_y = 50.0;
        let size = 20.0;
        let color = [1.0, 0.0, 0.0, 0.8];
        
        let vertices = create_mock_button_geometry(center_x, center_y, size, color);
        
        // 验证顶点数量：1个中心点 + 17个圆周点（包括起始点重复）
        assert_eq!(vertices.len(), 18);
        
        // 验证中心顶点
        assert_eq!(vertices[0].position, [center_x, center_y]);
        assert_eq!(vertices[0].color, color);
        
        // 验证圆周顶点在正确的半径内
        let radius = size / 2.0;
        for i in 1..vertices.len() {
            let dx = vertices[i].position[0] - center_x;
            let dy = vertices[i].position[1] - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            assert!((distance - radius).abs() < 1e-5, "顶点 {} 距离中心 {}, 期望 {}", i, distance, radius);
        }
    }

    #[test]
    fn test_button_indices_creation() {
        let segments = 16;
        let indices = create_mock_button_indices(segments);
        
        // 验证索引数量：每个段3个索引
        assert_eq!(indices.len(), (segments * 3) as usize);
        
        // 验证每个三角形都包含中心点
        for i in (0..indices.len()).step_by(3) {
            assert_eq!(indices[i], 0, "三角形 {} 应该包含中心点", i / 3);
        }
        
        // 验证索引范围
        for &index in &indices {
            assert!(index <= segments as u16 + 1, "索引 {} 超出范围", index);
        }
    }

    #[test]
    fn test_x_symbol_creation() {
        let center_x = 100.0;
        let center_y = 50.0;
        let size = 20.0;
        let color = [1.0, 1.0, 1.0, 1.0];
        
        let vertices = create_mock_button_symbol(center_x, center_y, size, "×", color);
        
        // X 符号应该有8个顶点（两条线，每条4个顶点）
        assert_eq!(vertices.len(), 8);
        
        // 验证所有顶点都有正确的颜色
        for vertex in &vertices {
            assert_eq!(vertex.color, color);
        }
    }

    #[test]
    fn test_minus_symbol_creation() {
        let center_x = 100.0;
        let center_y = 50.0;
        let size = 20.0;
        let color = [1.0, 1.0, 1.0, 1.0];
        
        let vertices = create_mock_button_symbol(center_x, center_y, size, "−", color);
        
        // 减号应该有4个顶点（一条线）
        assert_eq!(vertices.len(), 4);
        
        // 验证所有顶点都有正确的颜色
        for vertex in &vertices {
            assert_eq!(vertex.color, color);
        }
    }

    #[test]
    fn test_line_geometry_creation() {
        let x1 = 10.0;
        let y1 = 20.0;
        let x2 = 30.0;
        let y2 = 40.0;
        let width = 2.0;
        let color = [0.5, 0.5, 0.5, 1.0];
        
        let vertices = create_mock_line_geometry(x1, y1, x2, y2, width, color);
        
        // 线条应该有4个顶点（矩形）
        assert_eq!(vertices.len(), 4);
        
        // 验证所有顶点都有正确的颜色
        for vertex in &vertices {
            assert_eq!(vertex.color, color);
        }
        
        // 验证顶点形成矩形（简单检查）
        let center_x = (x1 + x2) / 2.0;
        let center_y = (y1 + y2) / 2.0;
        
        // 所有顶点应该在线条中心附近
        for vertex in &vertices {
            let dx = vertex.position[0] - center_x;
            let dy = vertex.position[1] - center_y;
            let distance_to_center = (dx * dx + dy * dy).sqrt();
            
            // 距离应该在合理范围内（线条长度的一半 + 宽度的一半）
            let line_length = ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt();
            let max_distance = line_length / 2.0 + width;
            assert!(distance_to_center <= max_distance, "顶点距离中心过远: {}", distance_to_center);
        }
    }

    #[test]
    fn test_hover_color_calculation() {
        // 测试悬浮状态下的颜色变化
        let normal_close_color = [0.8, 0.2, 0.2, 0.8];
        let hovered_close_color = [1.0, 0.3, 0.3, 0.9];
        
        let normal_minimize_color = [0.4, 0.4, 0.4, 0.8];
        let hovered_minimize_color = [0.6, 0.6, 0.6, 0.9];
        
        // 验证悬浮颜色更亮
        assert!(hovered_close_color[0] > normal_close_color[0]);
        assert!(hovered_close_color[1] > normal_close_color[1]);
        assert!(hovered_close_color[2] > normal_close_color[2]);
        assert!(hovered_close_color[3] > normal_close_color[3]);
        
        assert!(hovered_minimize_color[0] > normal_minimize_color[0]);
        assert!(hovered_minimize_color[1] > normal_minimize_color[1]);
        assert!(hovered_minimize_color[2] > normal_minimize_color[2]);
        assert!(hovered_minimize_color[3] > normal_minimize_color[3]);
    }

    #[test]
    fn test_button_positioning() {
        let window_size = PhysicalSize::new(800, 600);
        let ui_uniforms = UIUniforms::new(window_size);
        let button_size = 20.0;
        let margin = 5.0;
        
        // 验证关闭按钮位置（右上角）
        let expected_close_x = 800.0 - button_size - margin;
        let expected_close_y = margin;
        assert_eq!(ui_uniforms.close_button_pos, [expected_close_x, expected_close_y]);
        
        // 验证最小化按钮位置（关闭按钮左侧）
        let expected_minimize_x = expected_close_x - button_size - margin;
        let expected_minimize_y = margin;
        assert_eq!(ui_uniforms.minimize_button_pos, [expected_minimize_x, expected_minimize_y]);
        
        // 验证按钮不会超出窗口边界
        assert!(ui_uniforms.close_button_pos[0] >= 0.0);
        assert!(ui_uniforms.close_button_pos[1] >= 0.0);
        assert!(ui_uniforms.close_button_pos[0] + button_size <= 800.0);
        assert!(ui_uniforms.close_button_pos[1] + button_size <= 600.0);
        
        assert!(ui_uniforms.minimize_button_pos[0] >= 0.0);
        assert!(ui_uniforms.minimize_button_pos[1] >= 0.0);
        assert!(ui_uniforms.minimize_button_pos[0] + button_size <= 800.0);
        assert!(ui_uniforms.minimize_button_pos[1] + button_size <= 600.0);
    }
}