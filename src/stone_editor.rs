use macroquad::prelude::*;
use crate::basic_structs::Vector2D;
use crate::bezier::BezierInfo;
use macroquad::ui::{hash, root_ui, widgets};

// 对应 UML 中的编辑状态
#[derive(PartialEq)]
pub enum EditorMode {
    Menu,
    BezierDrawing,
    FreehandDrawing,
    Finished,
}

// 用于存储编辑器产生的数据
pub struct StoneBlueprint {
    pub points: Vec<Vector2D>, // 最终的轮廓点 (f64)
    pub thickness: f64,        // 厚度
    pub name: String,
}

pub struct StoneEditor {
    pub mode: EditorMode,
    pub thickness_input: String, // 用于UI输入的临时字符串

    // 贝塞尔模式数据
    bezier_control_points: Vec<Vector2D>,

    // 手绘模式数据
    freehand_points: Vec<Vector2D>,

    // 最终生成的蓝图
    pub result: Option<StoneBlueprint>,
}

impl StoneEditor {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Menu,
            thickness_input: "1.0".to_string(),
            bezier_control_points: Vec::new(),
            freehand_points: Vec::new(),
            result: None,
        }
    }

    pub async fn run(&mut self) {
        loop {
            clear_background(BLACK);

            match self.mode {
                EditorMode::Menu => self.draw_menu(),
                EditorMode::BezierDrawing => self.update_bezier(),
                EditorMode::FreehandDrawing => self.update_freehand(),
                EditorMode::Finished => break, // 退出编辑器循环
            }

            // 绘制通用的 UI (比如厚度输入)
            if self.mode != EditorMode::Finished && self.mode != EditorMode::Menu {
                self.draw_common_ui();
            }

            next_frame().await
        }
    }

    // --- 菜单界面 ---
    fn draw_menu(&mut self) {
        draw_text("STONE GENERATOR", 200.0, 400.0, 200.0, WHITE);

        // 简单的按钮逻辑
        if root_ui().button(vec2(500.0, 1000.0), "Mode: Bezier Curve") {
            self.mode = EditorMode::BezierDrawing;
            self.bezier_control_points.clear();
        }

        if root_ui().button(vec2(500.0, 1500.0), "Mode: Freehand Draw") {
            self.mode = EditorMode::FreehandDrawing;
            self.freehand_points.clear();
        }
    }

    // --- 通用 UI (厚度 & 完成) ---
    fn draw_common_ui(&mut self) {
        draw_text("Inputs:", 20.0, screen_height() - 100.0, 20.0, GRAY);

        // 厚度输入
        root_ui().label(Some(vec2(20.0, screen_height() - 70.0)), "Thickness:");

        // 使用 widgets::InputText 来支持自定义位置 (.position)
        widgets::InputText::new(hash!()) // hash!() 生成唯一ID
            .position(vec2(100.0, screen_height() - 75.0))
            .ui(&mut root_ui(), &mut self.thickness_input);

        // 完成按钮
        if root_ui().button(vec2(screen_width() - 120.0, screen_height() - 50.0), "FINISH & BUILD") {
            self.finalize_stone();
        }

        // 返回菜单
        if root_ui().button(vec2(20.0, screen_height() - 40.0), "Back to Menu") {
            self.mode = EditorMode::Menu;
        }
    }

    // --- 贝塞尔模式逻辑 ---
    fn update_bezier(&mut self) {
        draw_text("Click to add control points. Press Enter to finish.", 20.0, 30.0, 20.0, WHITE);

        // 1. 处理输入
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            // 坐标系转换: 屏幕中心为原点，方便物理计算
            let world_pos = screen_to_world(mx, my);
            self.bezier_control_points.push(world_pos);
        }

        // 2. 绘制控制点和连线 (辅助线)
        for (i, p) in self.bezier_control_points.iter().enumerate() {
            let screen_pos = world_to_screen(*p);
            draw_circle(screen_pos.x, screen_pos.y, 5.0, RED);

            if i > 0 {
                let prev = world_to_screen(self.bezier_control_points[i - 1]);
                draw_line(prev.x, prev.y, screen_pos.x, screen_pos.y, 1.0, DARKGRAY);
            }
        }

        // 3. 实时计算并绘制贝塞尔曲线
        // 注意：这里调用了你的 bezier.rs 逻辑
        if self.bezier_control_points.len() > 1 {
            // 克隆用于计算
            let info = BezierInfo::new("temp".to_string(), self.bezier_control_points.clone());
            let curve_points = info.get_polyline_points();

            // 绘制曲线
            for i in 0..curve_points.len() - 1 {
                let p1 = world_to_screen(curve_points[i]);
                let p2 = world_to_screen(curve_points[i+1]);
                draw_line(p1.x, p1.y, p2.x, p2.y, 2.0, YELLOW);
            }
        }
    }

    // --- 手绘模式逻辑 ---
    fn update_freehand(&mut self) {
        draw_text("Hold Left Click to draw.", 20.0, 30.0, 20.0, WHITE);

        // 1. 处理输入 (按住鼠标记录点)
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let world_pos = screen_to_world(mx, my);

            // 简单的去重，避免点太密集
            if let Some(last) = self.freehand_points.last() {
                let dist = ((last.x - world_pos.x).powi(2) + (last.y - world_pos.y).powi(2)).sqrt();
                if dist > 1.0 { // 最小间距
                    self.freehand_points.push(world_pos);
                }
            } else {
                self.freehand_points.push(world_pos);
            }
        }

        // 2. 绘制路径
        for i in 0..self.freehand_points.len().saturating_sub(1) {
            let p1 = world_to_screen(self.freehand_points[i]);
            let p2 = world_to_screen(self.freehand_points[i+1]);
            draw_line(p1.x, p1.y, p2.x, p2.y, 2.0, GREEN);
        }
    }

    // --- 生成最终数据 ---
    fn finalize_stone(&mut self) {
        let thickness: f64 = self.thickness_input.parse().unwrap_or(1.0);

        let final_points = match self.mode {
            EditorMode::BezierDrawing => {
                // 重新计算一次以确保精度
                let info = BezierInfo::new("final".to_string(), self.bezier_control_points.clone());
                info.polyline_points // 这里需要把字段设为 public 或者有 getter
            },
            EditorMode::FreehandDrawing => self.freehand_points.clone(),
            _ => Vec::new(),
        };

        if !final_points.is_empty() {
            self.result = Some(StoneBlueprint {
                points: final_points,
                thickness,
                name: "CustomStone".to_string(),
            });
            self.mode = EditorMode::Finished;
        }
    }
}

// --- 辅助函数：坐标转换 ---
// 屏幕坐标 (Top-Left 0,0) -> 物理世界坐标 (Center 0,0, f64)
fn screen_to_world(mx: f32, my: f32) -> Vector2D {
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;
    // 简单的平移，这里假设 1 pixel = 1 unit (你可以添加缩放)
    Vector2D {
        x: (mx - center_x) as f64,
        y: (center_y - my) as f64, // Y轴反转，物理世界通常Y向上
    }
}

// 物理世界坐标 (f64) -> 屏幕坐标 (f32)
fn world_to_screen(v: Vector2D) -> Vec2 {
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;
    vec2(
        center_x + v.x as f32,
        center_y - v.y as f32,
    )
}