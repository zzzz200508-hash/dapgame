use macroquad::prelude::*;
use crate::basic_structs::Vector2D; // [修正] 不再需要 Vector3D
use crate::bezier::BezierInfo;
use crate::physics::simulation::StoneInfo; // 假设 StoneInfo 现在使用 2D 向量

// 对应 UML 中的编辑状态
#[derive(PartialEq, Clone, Copy)]
pub enum EditorMode {
    Menu,
    BezierDrawing,
    FreehandDrawing,
    Preview,
    SetInitialConditions, // 初始条件设置
    Finished,
}

// 用于存储编辑器产生的数据
pub struct StoneBlueprint {
    pub points: Vec<Vector2D>, // 最终的轮廓点 (f64)
    pub thickness: f64,        // 厚度
    pub name: String,
}

// [修改] 辅助结构, 从 Vec3Input 变为 Vec2Input
#[derive(Clone)]
struct Vec2Input {
    x: String,
    y: String,
}

impl Vec2Input {
    fn new(x: &str, y: &str) -> Self {
        Self {
            x: x.to_string(),
            y: y.to_string(),
        }
    }
}

pub struct StoneEditor {
    pub mode: EditorMode,
    pub thickness_input: String,

    // 统一的文本输入状态
    active_input_id: Option<String>,

    // 贝塞尔模式数据
    bezier_control_points: Vec<Vector2D>,

    // 手绘模式数据
    freehand_points: Vec<Vector2D>,

    // 预览和状态管理
    previous_mode: EditorMode,
    preview_points: Vec<Vector2D>,
    self_intersection_warning: bool,

    // 阶段性存储
    blueprint_buffer: Option<StoneBlueprint>,

    // [修改] y0 (初始条件) 的输入
    y0_position: Vec2Input, // 变为 2D
    y0_velocity: Vec2Input, // 变为 2D
    y0_angle: String,         // 变为 1D
    y0_angular_velocity: String, // 变为 1D

    // 最终生成的蓝图
    pub result: Option<(StoneBlueprint, StoneInfo)>,
}

impl StoneEditor {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Menu,
            thickness_input: "1.0".to_string(),
            active_input_id: None,
            bezier_control_points: Vec::new(),
            freehand_points: Vec::new(),
            previous_mode: EditorMode::Menu,
            preview_points: Vec::new(),
            self_intersection_warning: false,
            blueprint_buffer: None,
            // [修改] y0 默认值
            y0_position: Vec2Input::new("0.0", "0.2"),    // 变为 2D
            y0_velocity: Vec2Input::new("10.0", "0.0"),  // 变为 2D
            y0_angle: "5.0".to_string(),                 // 变为 1D (5 度攻角)
            y0_angular_velocity: "30.0".to_string(),     // 变为 1D (绕 Z 轴旋转)

            result: None,
        }
    }

    pub async fn run(&mut self) {
        loop {
            clear_background(BLACK);

            // 统一的点击和键盘输入处理
            // 1. (全局) 鼠标点击默认取消所有输入框的焦点
            if is_mouse_button_pressed(MouseButton::Left) {
                self.active_input_id = None;
            }

            // 2. (全局) 键盘输入只写入当前激活的输入框
            self.handle_keyboard_input();

            // 3. 绘制当前状态的 UI (UI 内部会覆盖 active_input_id)
            match self.mode {
                EditorMode::Menu => self.draw_menu(),
                EditorMode::BezierDrawing => self.update_bezier(),
                EditorMode::FreehandDrawing => self.update_freehand(),
                EditorMode::Preview => self.draw_preview(),
                EditorMode::SetInitialConditions => self.draw_initial_conditions_ui(),
                EditorMode::Finished => break,
            }

            // 绘制通用的 UI (比如厚度输入)
            if self.mode == EditorMode::BezierDrawing || self.mode == EditorMode::FreehandDrawing {
                self.draw_common_ui();
            }

            next_frame().await
        }
    }

    // 键盘输入处理
    fn handle_keyboard_input(&mut self) {
        if self.active_input_id.is_none() { return; } // 没有激活的输入框

        // 1. 获取当前激活的 &mut String
        let s_mut_option: Option<&mut String> = match self.active_input_id.as_deref() {
            Some("thickness") => Some(&mut self.thickness_input),
            // [修改] 适配 2D
            // Position
            Some("pos_x") => Some(&mut self.y0_position.x),
            Some("pos_y") => Some(&mut self.y0_position.y),
            // Velocity
            Some("vel_x") => Some(&mut self.y0_velocity.x),
            Some("vel_y") => Some(&mut self.y0_velocity.y),
            // Angle (1D)
            Some("ang") => Some(&mut self.y0_angle),
            // Angular Velocity (1D)
            Some("ang_vel") => Some(&mut self.y0_angular_velocity),
            _ => None,
        };

        // 2. 将键盘事件写入
        if let Some(s_mut) = s_mut_option {
            while let Some(c) = get_char_pressed() {
                match c {
                    '\u{0008}' => { s_mut.pop(); }, // 退格键
                    '\r' | '\n' => { self.active_input_id = None; break; }, // 回车键取消焦点
                    // 允许数字、小数点和负号
                    c if c.is_digit(10) || c == '.' || (c == '-' && s_mut.is_empty()) => {
                        s_mut.push(c);
                    },
                    _ => {}
                }
            }
        }
    }

    // 绘制菜单
    fn draw_menu(&mut self) {
        let title_font_size = 120.0;
        let title_text = "STONE GENERATOR";
        let text_dims = measure_text(title_text, None, title_font_size as u16, 1.0);

        draw_text(
            title_text,
            screen_width() / 2.0 - text_dims.width / 2.0,
            400.0,
            title_font_size,
            WHITE,
        );

        let btn_width = 500.0;
        let btn_height = 75.0;
        let btn_x = screen_width() / 2.0 - btn_width / 2.0;
        let btn_y1 = 1000.0;
        let btn_rect1 = Rect::new(btn_x, btn_y1, btn_width, btn_height);

        draw_rectangle_lines(btn_rect1.x, btn_rect1.y, btn_rect1.w, btn_rect1.h, 4.0, GRAY);
        draw_text_ex("Mode: Bezier Curve", btn_rect1.x + 20.0, btn_rect1.y + btn_rect1.h - 20.0,
                     TextParams { font_size: 48, ..Default::default() });

        let btn_y2 = 1100.0;
        let btn_rect2 = Rect::new(btn_x, btn_y2, btn_width, btn_height);

        draw_rectangle_lines(btn_rect2.x, btn_rect2.y, btn_rect2.w, btn_rect2.h, 4.0, GRAY);
        draw_text_ex("Mode: Freehand Draw", btn_rect2.x + 20.0, btn_rect2.y + btn_rect2.h - 20.0,
                     TextParams { font_size: 48, ..Default::default() });

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if btn_rect1.contains(vec2(mx, my)) {
                self.mode = EditorMode::BezierDrawing;
                self.bezier_control_points.clear();
                self.active_input_id = None;
            }
            if btn_rect2.contains(vec2(mx, my)) {
                self.mode = EditorMode::FreehandDrawing;
                self.freehand_points.clear();
                self.active_input_id = None;
            }
        }
    }

    // 通用 UI, 只绘制厚度
    fn draw_common_ui(&mut self) {
        let font_size = 48.0;
        let base_y = screen_height() - 250.0;
        let padding = 30.0;
        let control_height = 70.0;

        draw_text("Inputs:", 50.0, base_y - 70.0, font_size + 10.0, GRAY);

        // --- 厚度输入行 ---
        draw_text_ex("Thickness:", 50.0, base_y + control_height - 20.0,
                     TextParams { font_size: font_size as u16, ..Default::default() });

        let input_width = 200.0;
        let input_x = 50.0 + 300.0;
        let input_rect = Rect::new(input_x, base_y, input_width, control_height);

        // [FIX] 调用新的自由函数
        if draw_text_input_box(
            &self.thickness_input,
            input_rect,
            "thickness",
            &self.active_input_id,
            font_size as u16
        ) {
            self.active_input_id = Some("thickness".to_string());
        }

        let cm_label_x = input_x + input_width + padding;
        draw_text_ex("cm", cm_label_x, base_y + control_height - 20.0,
                     TextParams { font_size: font_size as u16, ..Default::default() });

        let btn_width = 400.0;
        let btn_height = 75.0;

        let btn_finish_rect = Rect::new(screen_width() - 450.0, screen_height() - 120.0, btn_width, btn_height);
        draw_rectangle(btn_finish_rect.x, btn_finish_rect.y, btn_finish_rect.w, btn_finish_rect.h, DARKGRAY);
        draw_text_ex("FINISH & BUILD", btn_finish_rect.x + 20.0, btn_finish_rect.y + btn_finish_rect.h - 25.0,
                     TextParams { font_size: font_size as u16, color: WHITE, ..Default::default() });

        let btn_back_rect = Rect::new(50.0, screen_height() - 120.0, btn_width, btn_height);
        draw_rectangle(btn_back_rect.x, btn_back_rect.y, btn_back_rect.w, btn_back_rect.h, DARKGRAY);
        draw_text_ex("Back to Menu", btn_back_rect.x + 20.0, btn_back_rect.y + btn_back_rect.h - 25.0,
                     TextParams { font_size: font_size as u16, color: WHITE, ..Default::default() });

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let mouse_pos = vec2(mx, my);

            if btn_finish_rect.contains(mouse_pos) {
                self.finalize_stone();
                self.active_input_id = None;
            }
            if btn_back_rect.contains(mouse_pos) {
                self.mode = EditorMode::Menu;
                self.active_input_id = None;
            }
        }
    }

    // 贝塞尔模式
    fn update_bezier(&mut self) {
        draw_text("Click to add control points.", 20.0, 30.0, 40.0, WHITE);

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            // 避免点击 UI 区域
            if my < screen_height() - 300.0 {
                let world_pos = screen_to_world(mx, my);
                self.bezier_control_points.push(world_pos);
            }
        }

        for (i, p) in self.bezier_control_points.iter().enumerate() {
            let screen_pos = world_to_screen(*p);
            draw_circle(screen_pos.x, screen_pos.y, 10.0, RED);
            if i > 0 {
                let prev = world_to_screen(self.bezier_control_points[i - 1]);
                draw_line(prev.x, prev.y, screen_pos.x, screen_pos.y, 2.0, DARKGRAY);
            }
        }

        if self.bezier_control_points.len() > 1 {
            let info = BezierInfo::new("temp".to_string(), self.bezier_control_points.clone());
            let curve_points = info.get_polyline_points();
            for i in 0..curve_points.len() - 1 {
                let p1 = world_to_screen(curve_points[i]);
                let p2 = world_to_screen(curve_points[i+1]);
                draw_line(p1.x, p1.y, p2.x, p2.y, 4.0, YELLOW);
            }
        }
    }

    // 手绘模式
    fn update_freehand(&mut self) {
        draw_text("Hold Left Click to draw.", 20.0, 30.0, 40.0, WHITE);

        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if my < screen_height() - 300.0 {
                let world_pos = screen_to_world(mx, my);
                if let Some(last) = self.freehand_points.last() {
                    let dist_sq = (last.x - world_pos.x).powi(2) + (last.y - world_pos.y).powi(2);
                    if dist_sq > 5.0 * 5.0 {
                        self.freehand_points.push(world_pos);
                    }
                } else {
                    self.freehand_points.push(world_pos);
                }
            }
        }

        for i in 0..self.freehand_points.len().saturating_sub(1) {
            let p1 = world_to_screen(self.freehand_points[i]);
            let p2 = world_to_screen(self.freehand_points[i+1]);
            draw_line(p1.x, p1.y, p2.x, p2.y, 4.0, GREEN);
        }
    }

    // 预览绘制
    fn draw_preview(&mut self) {
        let font_size = 48.0;
        let screen_points: Vec<Vec2> = self.preview_points.iter()
            .map(|p| world_to_screen(*p))
            .collect();

        if screen_points.len() > 1 {
            for i in 0..screen_points.len() - 1 {
                let p1 = screen_points[i];
                let p2 = screen_points[i+1];
                draw_line(p1.x, p1.y, p2.x, p2.y, 4.0, GREEN);
            }
            if let (Some(first), Some(last)) = (screen_points.first(), screen_points.last()) {
                draw_line(first.x, first.y, last.x, last.y, 4.0, GREEN);
            }
        }

        let title_text = "PREVIEW";
        let text_dims = measure_text(title_text, None, 60, 1.0);
        draw_text(title_text, screen_width() / 2.0 - text_dims.width / 2.0, 80.0, 60.0, WHITE);

        if self.self_intersection_warning {
            let warn_text = "Warning: Shape self-intersects!";
            let warn_text_2 = "This may cause physics issues.";
            let warn_dims = measure_text(warn_text, None, 40, 1.0);
            let warn_dims_2 = measure_text(warn_text_2, None, 40, 1.0);
            draw_text(warn_text, screen_width() / 2.0 - warn_dims.width / 2.0, 150.0, 40.0, RED);
            draw_text(warn_text_2, screen_width() / 2.0 - warn_dims_2.width / 2.0, 200.0, 40.0, RED);
            let warn_text_3 = "Multiple regions are not supported.";
            let warn_dims_3 = measure_text(warn_text_3, None, 40, 1.0);
            draw_text(warn_text_3, screen_width() / 2.0 - warn_dims_3.width / 2.0, 250.0, 40.0, RED);
        }

        let btn_width = 400.0;
        let btn_height = 75.0;

        let btn_confirm_rect = Rect::new(screen_width() - 450.0, screen_height() - 120.0, btn_width, btn_height);
        draw_rectangle(btn_confirm_rect.x, btn_confirm_rect.y, btn_confirm_rect.w, btn_confirm_rect.h, DARKGREEN);
        draw_text_ex("CONFIRM", btn_confirm_rect.x + 20.0, btn_confirm_rect.y + btn_confirm_rect.h - 25.0,
                     TextParams { font_size: font_size as u16, color: WHITE, ..Default::default() });

        let btn_back_rect = Rect::new(50.0, screen_height() - 120.0, btn_width, btn_height);
        draw_rectangle(btn_back_rect.x, btn_back_rect.y, btn_back_rect.w, btn_back_rect.h, DARKGRAY);
        draw_text_ex("Go Back (Edit)", btn_back_rect.x + 20.0, btn_back_rect.y + btn_back_rect.h - 25.0,
                     TextParams { font_size: font_size as u16, color: WHITE, ..Default::default() });

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let mouse_pos = vec2(mx, my);

            if btn_confirm_rect.contains(mouse_pos) {
                let thickness_cm: f64 = self.thickness_input.parse().unwrap_or(1.0);
                let thickness_meters = thickness_cm / 100.0;

                self.blueprint_buffer = Some(StoneBlueprint {
                    points: self.preview_points.clone(),
                    thickness: thickness_meters,
                    name: "CustomStone".to_string(),
                });
                self.mode = EditorMode::SetInitialConditions;
                self.active_input_id = None;
            }

            if btn_back_rect.contains(mouse_pos) {
                self.preview_points.clear();
                self.self_intersection_warning = false;
                self.mode = self.previous_mode;
                self.active_input_id = None;
            }
        }
    }

    // 初始条件设置 UI
    fn draw_initial_conditions_ui(&mut self) {
        let font_size = 48.0;
        let title_font_size = 60.0;
        let start_y = 150.0;
        let row_height = 100.0;
        let input_w = 200.0;
        let input_h = 70.0;
        let label_w = 400.0;
        let col_w = 250.0;
        let start_x = 100.0;

        // 侧边预览窗口
        self.draw_side_screen_preview();

        // 标题
        let title_text = "SET INITIAL CONDITIONS (2D)";
        let text_dims = measure_text(title_text, None, title_font_size as u16, 1.0);
        draw_text(title_text, screen_width() / 2.0 - text_dims.width / 2.0, 80.0, title_font_size, WHITE);

        let mut y = start_y;

        // --- [FIX] 存储从自由函数返回的点击ID ---
        let mut clicked_id: Option<String> = None;

        // --- 1. Position ---
        // [FIX] 调用自由函数并传递 active_id
        let id = draw_vec2_input_row(
            "Position (m)",
            &self.y0_position,
            "pos",
            y,
            start_x, label_w, col_w, input_w, input_h, font_size as u16,
            &self.active_input_id
        );
        if id.is_some() { clicked_id = id; }
        y += row_height;

        // --- 2. Velocity (m/s) ---
        let id = draw_vec2_input_row(
            "Velocity (m/s)",
            &self.y0_velocity,
            "vel",
            y,
            start_x, label_w, col_w, input_w, input_h, font_size as u16,
            &self.active_input_id
        );
        if id.is_some() { clicked_id = id; }
        y += row_height;

        // --- 3. Angle (deg) ---
        let id = draw_f64_input_row(
            "Angle (deg)",
            &self.y0_angle,
            "ang",
            y,
            start_x, label_w, input_w, input_h, font_size as u16,
            &self.active_input_id
        );
        if id.is_some() { clicked_id = id; }
        y += row_height;

        // --- 4. Angular Velocity (rad/s) ---
        let id = draw_f64_input_row(
            "Ang. Vel (rad/s)",
            &self.y0_angular_velocity,
            "ang_vel",
            y,
            start_x, label_w, input_w, input_h, font_size as u16,
            &self.active_input_id
        );
        if id.is_some() { clicked_id = id; }

        // --- [FIX] 在所有绘制完成后才更新 self ---
        if let Some(id_str) = clicked_id {
            self.active_input_id = Some(id_str);
        }

        // --- 按钮 ---
        let btn_width = 500.0;
        let btn_height = 75.0;

        let btn_start_rect = Rect::new(screen_width() - (btn_width + 50.0), screen_height() - 120.0, btn_width, btn_height);
        draw_rectangle(btn_start_rect.x, btn_start_rect.y, btn_start_rect.w, btn_start_rect.h, DARKGREEN);
        draw_text_ex("START SIMULATION", btn_start_rect.x + 20.0, btn_start_rect.y + btn_start_rect.h - 25.0,
                     TextParams { font_size: font_size as u16, color: WHITE, ..Default::default() });

        let btn_back_rect = Rect::new(50.0, screen_height() - 120.0, btn_width, btn_height);
        draw_rectangle(btn_back_rect.x, btn_back_rect.y, btn_back_rect.w, btn_back_rect.h, DARKGRAY);
        draw_text_ex("Back (Preview)", btn_back_rect.x + 20.0, btn_back_rect.y + btn_back_rect.h - 25.0,
                     TextParams { font_size: font_size as u16, color: WHITE, ..Default::default() });

        // --- 点击检测 ---
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let mouse_pos = vec2(mx, my);

            if btn_start_rect.contains(mouse_pos) {
                self.finish_and_build_y0();
                self.active_input_id = None;
            }

            if btn_back_rect.contains(mouse_pos) {
                self.mode = EditorMode::Preview;
                self.active_input_id = None;
            }
        }
    }

    // 完成形状
    fn finalize_stone(&mut self) {
        self.previous_mode = self.mode;

        let mut points_to_process = match self.mode {
            EditorMode::BezierDrawing => self.bezier_control_points.clone(),
            EditorMode::FreehandDrawing => self.freehand_points.clone(),
            _ => Vec::new(),
        };

        if points_to_process.len() < 2 { return; }

        let first = points_to_process.first().unwrap();
        let last = points_to_process.last().unwrap();
        let dist_sq = (first.x - last.x).powi(2) + (first.y - last.y).powi(2);

        if dist_sq > (0.02 * 0.02) {
            points_to_process.push(*first);
        }

        let final_points = match self.mode {
            EditorMode::BezierDrawing => {
                let info = BezierInfo::new("final".to_string(), points_to_process);
                info.polyline_points
            },
            EditorMode::FreehandDrawing => points_to_process,
            _ => Vec::new(),
        };

        if final_points.is_empty() { return; }

        let intersection_count = self.count_self_intersections(&final_points);
        self.self_intersection_warning = intersection_count >= 2;

        self.preview_points = final_points;
        self.mode = EditorMode::Preview;
    }

    // --- 几何辅助函数 (保持 &self) ---

    // 计算自相交次数
    fn count_self_intersections(&self, points: &[Vector2D]) -> usize {
        if points.len() < 4 { return 0; }
        let mut intersection_count = 0;
        let num_segments = points.len() - 1;

        for i in 0..num_segments {
            let p1 = points[i];
            let p2 = points[i + 1];

            for j in (i + 2)..num_segments {
                if i == 0 && j == num_segments - 1 {
                    continue;
                }
                let p3 = points[j];
                let p4 = points[j + 1];
                if self.line_segments_intersect(p1, p2, p3, p4) {
                    intersection_count += 1;
                }
            }
        }
        intersection_count
    }

    // 检查线段相交
    fn line_segments_intersect(&self, a: Vector2D, b: Vector2D, c: Vector2D, d: Vector2D) -> bool {
        let o1 = self.orientation(a, b, c);
        let o2 = self.orientation(a, b, d);
        let o3 = self.orientation(c, d, a);
        let o4 = self.orientation(c, d, b);
        if o1 != o2 && o3 != o4 { return true; }
        if o1 == 0 && self.on_segment(a, c, b) { return true; }
        if o2 == 0 && self.on_segment(a, d, b) { return true; }
        if o3 == 0 && self.on_segment(c, a, d) { return true; }
        if o4 == 0 && self.on_segment(c, b, d) { return true; }
        false
    }

    // 几何方向
    fn orientation(&self, p: Vector2D, q: Vector2D, r: Vector2D) -> i8 {
        let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
        if val.abs() < 1e-10 { return 0; }
        if val > 0.0 { 1 } else { 2 }
    }

    // 检查点是否在线段上
    fn on_segment(&self, p: Vector2D, q: Vector2D, r: Vector2D) -> bool {
        q.x <= f64::max(p.x, r.x) && q.x >= f64::min(p.x, r.x) &&
            q.y <= f64::max(p.y, r.y) && q.y >= f64::min(p.y, r.y)
    }

    // [FIX] 绘制侧边预览小屏幕 (改为 &self)
    fn draw_side_screen_preview(&self) {
        // 1. 定义预览区域
        let rect = Rect::new(screen_width() - 550.0, 150.0, 500.0, 400.0);
        let title = "Cross-section Preview";
        let font_size = 30;

        // 绘制边框和标题
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, GRAY);
        let title_dims = measure_text(title, None, font_size, 1.0);
        draw_text(title, rect.x + rect.w / 2.0 - title_dims.width / 2.0, rect.y + 40.0, font_size as f32, WHITE);

        // 2. 定义 "世界" 坐标
        let world_center_x = rect.x + rect.w / 2.0;
        let world_y_zero = rect.y + rect.h / 2.0 + 50.0; // 水面 (Y=0) 在屏幕上的 Y 坐标
        let world_scale = 300.0; // 预览中的缩放比例 (1 米 = 300 像素)

        // 3. 绘制水
        draw_line(rect.x, world_y_zero, rect.x + rect.w, world_y_zero, 2.0, BLUE);
        draw_text("Water (Y=0)", rect.x + 10.0, world_y_zero + 30.0, 24.0, BLUE);

        // 4. 解析当前输入值
        let parse = |s: &String| s.parse::<f64>().unwrap_or(0.0);
        let deg_to_rad = |deg: f64| deg * std::f64::consts::PI / 180.0;

        let pos_y = parse(&self.y0_position.y); // Y 坐标 (m)
        let angle_deg = parse(&self.y0_angle);  // 角度 (deg)
        let angle_rad = deg_to_rad(angle_deg);  // 角度 (rad)

        // 5. 计算石片在预览中的位置和朝向
        let stone_draw_y = world_y_zero - (pos_y * world_scale) as f32;
        let stone_len = 100.0; // 石片在预览中的固定长度

        // 计算旋转后的线段端点
        // 注意: macroquad 的 Y 轴向下, sin() 应该反转
        let cos_a = angle_rad.cos() as f32;
        let sin_a = angle_rad.sin() as f32;

        let p1_x = world_center_x - cos_a * (stone_len / 2.0);
        let p1_y = stone_draw_y - sin_a * (stone_len / 2.0);

        let p2_x = world_center_x + cos_a * (stone_len / 2.0);
        let p2_y = stone_draw_y + sin_a * (stone_len / 2.0);

        // 6. 绘制石片
        draw_line(p1_x, p1_y, p2_x, p2_y, 5.0, YELLOW);

        // 绘制一个 "前" 标记
        draw_circle(p2_x, p2_y, 6.0, RED);
        draw_text_ex("Front", p2_x, p2_y - 15.0, TextParams { font_size: 20, color: RED, ..Default::default() });
    }

    // [修正] 最终构建 y0 (纯 2D)
    fn finish_and_build_y0(&mut self) {
        // 1. 辅助函数, 解析字符串
        let parse = |s: &String| s.parse::<f64>().unwrap_or(0.0);
        let deg_to_rad = |deg: f64| deg * std::f64::consts::PI / 180.0;

        // 2. [修正] 解析 2D 值
        let pos = Vector2D::new(
            parse(&self.y0_position.x),
            parse(&self.y0_position.y)
        );
        let vel = Vector2D::new(
            parse(&self.y0_velocity.x),
            parse(&self.y0_velocity.y)
        );

        // 2D 模拟, 角度和角速度是 f64
        // (角度输入是 度, 物理计算需要 弧度)
        let ang = deg_to_rad(parse(&self.y0_angle));

        // (角速度输入已经是 rad/s)
        let ang_vel = parse(&self.y0_angular_velocity);

        // 3. 创建 y0 StoneInfo (假设 StoneInfo 是 2D 结构)
        //======
        //重点：生成y0
        //======
        let y0 = StoneInfo {
            position: pos,
            velocity: vel,
            angle: Vector2D::new(ang, 0.0),
            angle_velocity: Vector2D::new(0.0,ang_vel),
        };

        // 4. 合并 blueprint 和 y0
        if let Some(blueprint) = self.blueprint_buffer.take() { // .take() 会取出 Some(T), 留下 None
            self.result = Some((blueprint, y0));
            self.mode = EditorMode::Finished;
        } else {
            // 这是一个错误状态, 意味着 blueprint_buffer 是 None
            println!("Error: Blueprint buffer was empty when trying to build y0!");
            self.mode = EditorMode::Menu; // 重置
        }
    }
}

// --- [FIX] UI 辅助绘制函数 (移出 impl 块) ---

// 绘制一个可点击的文本输入框
// [FIX] 不再是 &mut self 的方法
// [FIX] 返回 bool (是否被点击)
fn draw_text_input_box(
    text: &String,
    rect: Rect,
    id: &str,
    active_id: &Option<String>,
    font_size: u16
) -> bool {
    let mut clicked = false;
    // 检查是否被点击
    if is_mouse_button_pressed(MouseButton::Left) {
        if rect.contains(mouse_position().into()) {
            clicked = true;
        }
    }

    let is_active = active_id.as_deref() == Some(id);

    // 绘制框
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0,
                         if is_active { YELLOW } else { GRAY });

    // 绘制文本
    let text_to_draw = if is_active {
        // 添加闪烁的光标 (简单实现)
        if (get_time() * 2.0).fract() > 0.5 {
            format!("{}|", text)
        } else {
            text.clone()
        }
    } else {
        text.clone()
    };

    draw_text_ex(&text_to_draw, rect.x + 10.0, rect.y + rect.h - (rect.h - font_size as f32) / 2.0 - 5.0,
                 TextParams { font_size, color: WHITE, ..Default::default() });

    clicked // 返回点击状态
}

// 绘制一整行 Vec2 输入 (Label + X, Y inputs)
// [FIX] 不再是 &mut self 的方法
// [FIX] 返回 Option<String> (被点击的 ID)
fn draw_vec2_input_row(
    label_text: &str,
    input_data: &Vec2Input,
    id_prefix: &str,
    y: f32,
    start_x: f32,
    label_w: f32,
    col_w: f32,
    input_w: f32,
    input_h: f32,
    font_size: u16,
    active_id: &Option<String>
) -> Option<String> {
    let text_y_offset = input_h - (input_h - font_size as f32) / 2.0 - 5.0;

    // 1. 绘制行标签
    draw_text_ex(label_text, start_x, y + text_y_offset,
                 TextParams { font_size, color: WHITE, ..Default::default() });

    let mut x = start_x + label_w;

    // 2. 绘制 X, Y 标签和输入框
    for (component_label, component_id_suffix, component_data) in [
        ("X:", "x", &input_data.x),
        ("Y:", "y", &input_data.y),
    ] {
        let component_id = format!("{}_{}", id_prefix, component_id_suffix);

        // 标签 (X:, Y:)
        draw_text_ex(component_label, x, y + text_y_offset,
                     TextParams { font_size, color: GRAY, ..Default::default() });

        // 输入框
        let input_rect = Rect::new(x + 50.0, y, input_w, input_h);
        if draw_text_input_box(component_data, input_rect, &component_id, active_id, font_size) {
            return Some(component_id); // [FIX] 返回被点击的 ID
        }

        x += col_w;
    }

    None // [FIX] 没有点击
}

// 绘制一整行 f64 (单个) 输入
// [FIX] 不再是 &mut self 的方法
// [FIX] 返回 Option<String> (被点击的 ID)
fn draw_f64_input_row(
    label_text: &str,
    input_data: &String,
    id: &str,
    y: f32,
    start_x: f32,
    label_w: f32,
    input_w: f32,
    input_h: f32,
    font_size: u16,
    active_id: &Option<String>
) -> Option<String> {
    let text_y_offset = input_h - (input_h - font_size as f32) / 2.0 - 5.0;

    // 1. 绘制行标签
    draw_text_ex(label_text, start_x, y + text_y_offset,
                 TextParams { font_size, color: WHITE, ..Default::default() });

    let x = start_x + label_w;

    // 2. 绘制输入框
    let input_rect = Rect::new(x, y, input_w, input_h);
    if draw_text_input_box(input_data, input_rect, id, active_id, font_size) {
        return Some(id.to_string()); // [FIX] 返回被点击的 ID
    }

    None // [FIX] 没有点击
}


// --- 辅助函数：坐标转换 (保持不变) ---
// 屏幕坐标 (Top-Left 0,0) -> 物理世界坐标 (Center 0,0, f64, 米)
fn screen_to_world(mx: f32, my: f32) -> Vector2D {
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    // 假设屏幕上的 1000 像素 = 1 米
    let scale = 8000.0;

    Vector2D {
        x: (mx - center_x) as f64 / scale,
        y: (center_y - my) as f64 / scale, // Y轴反转, 物理世界Y向上
    }
}

// 物理世界坐标 (f64, 米) -> 屏幕坐标 (f32)
fn world_to_screen(v: Vector2D) -> Vec2 {
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    let scale = 8000.0; // 1 米 对应 1000 像素

    vec2(
        center_x + (v.x * scale) as f32,
        center_y - (v.y * scale) as f32, // Y轴反转
    )
}