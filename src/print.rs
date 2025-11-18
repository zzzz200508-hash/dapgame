// 职责：接收2D物理模拟的状态，并在XY平面上将其可视化。

use macroquad::prelude::*;
use crate::physics::simulation::StoneInfo; // 你的 (2D) 状态向量
use crate::basic_structs::Vector2D;
use crate::stone_phy::StoneProperties; // 你的物理属性结构体

/// # 2D 模拟渲染器
///
/// 负责在 2D (XY) 平面中绘制模拟。
/// 假定物理世界中 Y 轴向上，X 轴向右。
pub struct SimulationRenderer {
    /// 存储所有历史帧，用于绘制轨迹
    trajectory: Vec<StoneInfo>,
    /// 石片的物理和几何属性 (质心系)
    stone_props: StoneProperties,

    current_frame: usize,
    is_playing: bool,

    /// 视觉缩放比例 (像素/米)
    scale: f64,
    /// 世界坐标 (0, 0) 在屏幕上的像素位置
    world_origin_on_screen: Vec2,

    // 摄像机平移状态
    is_panning: bool,
    last_mouse_pos: Vec2,
}

impl SimulationRenderer {
    /// 创建一个新的渲染器
    ///
    /// - `stone_props`: 从 stone_factory 计算得出的石片物理属性。
    /// - `scale`: 初始缩放比例 (例如: 1000.0 像素/米)
    pub fn new(stone_props: StoneProperties, scale: f64) -> Self {
        Self {
            trajectory: Vec::new(),
            stone_props,
            current_frame: 0,
            is_playing: true, // 默认自动播放
            scale,
            // 默认将 (0,0) 放在屏幕左 1/4, 垂直 3/4 的位置
            world_origin_on_screen: vec2(screen_width() / 4.0, screen_height() * 0.75),
            is_panning: false,
            last_mouse_pos: Vec2::ZERO,
        }
    }

    /// (主循环调用) 添加一个新的状态帧
    pub fn add_state(&mut self, state: StoneInfo) {
        self.trajectory.push(state);
    }

    /// (主循环调用) 更新并绘制当前帧
    pub fn draw_and_update(&mut self) {
        clear_background(Color::from_rgba(10, 20, 35, 100)); // 深蓝色背景

        // 1. (新) 处理摄像机控制 (平移和缩放)
        self.handle_view_controls();

        // 2. 绘制静态元素 (网格, 水面)
        self.draw_grid_and_axes();
        self.draw_water_surface(); // 绘制 Y=0 的水面

        // 3. 绘制动态元素 (轨迹, 石块)
        if !self.trajectory.is_empty() {
            // 绘制轨迹线
            self.draw_trajectory_path();

            // 获取当前帧 (如果暂停则固定, 播放则推进)
            let state_to_draw = &self.trajectory[self.current_frame];
            self.draw_stone(state_to_draw);

            self.draw_rotation_preview(state_to_draw);

            if self.is_playing && self.current_frame < self.trajectory.len() - 1 {
                self.current_frame += 1;
            }
        } else {
            draw_text(
                "等待模拟数据...",
                screen_width() / 2.0 - 150.0,
                screen_height() / 2.0,
                30.0,
                WHITE,
            );
        }

        // 4. 绘制 UI 信息
        self.draw_info_panel();
    }

    // --- 核心绘制函数 ---

    /// 绘制石片
    fn draw_stone(&self, state: &StoneInfo) {
        // 1. 获取石片的基准形状 (位于质心系, 0,0)
        let base_outline = &self.stone_props.outline_com;
        if base_outline.is_empty() { return; }

        // 2. 获取当前状态
        let world_pos = state.position;
        let tilt_angle = state.angle.x; // 物理中的俯仰角 (angle.y 是自转, 2D 中不可见)
        let cos_a = tilt_angle.cos();
        let sin_a = tilt_angle.sin();

        // 3. 转换并绘制轮廓
        let screen_points: Vec<Vec2> = base_outline.iter().map(|&local_point| {
            // 3a. 应用旋转 (绕质心)
            let rotated_x = local_point.x * cos_a - local_point.y * sin_a;
            let rotated_y = local_point.x * sin_a + local_point.y * cos_a;

            // 3b. 应用平移 (到世界坐标)
            let world_point = Vector2D {
                x: rotated_x + world_pos.x,
                y: rotated_y + world_pos.y,
            };

            // 3c. 转换到屏幕坐标
            self.world_to_screen(world_point)
        }).collect();

        // 4. 绘制轮廓线
        if screen_points.len() > 2 {
            for i in 0..screen_points.len() {
                let p1 = screen_points[i];
                let p2 = screen_points[(i + 1) % screen_points.len()];
                draw_line(p1.x, p1.y, p2.x, p2.y, 2.0, YELLOW);
            }
        }

        // 5. 绘制石片质心
        let com_screen = self.world_to_screen(world_pos);
        draw_circle(com_screen.x, com_screen.y, 3.0, RED);
    }

    /// 绘制一个显示“自转”的俯视小窗
    fn draw_rotation_preview(&self, state: &StoneInfo) {
        // 1. 定义小窗的位置和大小
        let rect = Rect::new(20.0, 100.0, 200.0, 200.0);
        let center = rect.center();

        // 2. 绘制边框和标题
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, GRAY);
        draw_text("Spin (Top-Down)", rect.x + 30.0, rect.y + 30.0, 20.0, WHITE);

        // 3. 获取石片数据
        let base_outline = &self.stone_props.outline_com;
        if base_outline.is_empty() { return; }

        // d_max 是距离的平方, 我们需要半径
        let max_radius = self.stone_props.d_max.sqrt();
        if max_radius < 1e-9 { return; } // 避免除以零

        // 4. 计算此小窗的本地缩放
        // (让石片占满 80% 的窗口, 留出边距)
        let local_scale = (rect.w * 0.8 / 2.0) as f64 / max_radius;

        // 5. 获取当前自转角度
        let spin_angle = state.angle.y; // 这对应于你的 "angle.y"
        let cos_spin = spin_angle.cos();
        let sin_spin = spin_angle.sin();

        // 6. 转换并绘制轮廓
        let screen_points: Vec<Vec2> = base_outline.iter().map(|&local_point| {
            // 6a. 应用自转 (绕 0,0)
            let rotated_x = local_point.x * cos_spin - local_point.y * sin_spin;
            let rotated_y = local_point.x * sin_spin + local_point.y * cos_spin;

            // 6b. 缩放并平移到小窗中心
            vec2(
                center.x + (rotated_x * local_scale) as f32,
                center.y - (rotated_y * local_scale) as f32 // Y 轴反转
            )
        }).collect();

        // 7. 绘制轮廓线
        if screen_points.len() > 2 {
            for i in 0..screen_points.len() {
                let p1 = screen_points[i];
                let p2 = screen_points[(i + 1) % screen_points.len()];
                draw_line(p1.x, p1.y, p2.x, p2.y, 1.0, YELLOW);
            }
        }
    }

    /// 绘制轨迹（所有历史位置）
    fn draw_trajectory_path(&self) {
        if self.trajectory.len() < 2 { return; }

        // 只绘制到当前帧
        let end_index = (self.current_frame + 1).min(self.trajectory.len());
        for i in 0..(end_index.saturating_sub(1)) {
            let p1 = self.world_to_screen(self.trajectory[i].position);
            let p2 = self.world_to_screen(self.trajectory[i + 1].position);
            draw_line(p1.x, p1.y, p2.x, p2.y, 1.0, YELLOW);
        }
    }

    /// 绘制 Y=0 的水面线
    fn draw_water_surface(&self) {
        let water_y_screen = self.world_to_screen(Vector2D::new(0.0, 0.0)).y;

        // 绘制水面
        draw_line(0.0, water_y_screen, screen_width(), water_y_screen, 2.0, BLUE);
        // 绘制水下区域 (填充)
        draw_rectangle(0.0, water_y_screen, screen_width(), screen_height() - water_y_screen, Color::new(0.0, 0.2, 0.5, 0.3));

        draw_text("Water (Y=0)", 20.0, water_y_screen + 30.0, 20.0, LIGHTGRAY);
    }

    /// 绘制背景网格和 X/Y 轴
    fn draw_grid_and_axes(&self) {
        let grid_spacing = 50.0; // 50 像素的网格
        let grid_color = Color::new(1.0, 1.0, 1.0, 0.1);

        // 0,0 点的屏幕坐标
        let ox = self.world_origin_on_screen.x;
        let oy = self.world_origin_on_screen.y;

        // --- 绘制网格 ---
        // 垂直线 (X)
        let mut x = ox;
        while x < screen_width() { draw_line(x, 0.0, x, screen_height(), 1.0, grid_color); x += grid_spacing; }
        let mut x = ox - grid_spacing;
        while x > 0.0 { draw_line(x, 0.0, x, screen_height(), 1.0, grid_color); x -= grid_spacing; }

        // 水平线 (Y)
        let mut y = oy;
        while y < screen_height() { draw_line(0.0, y, screen_width(), y, 1.0, grid_color); y += grid_spacing; }
        let mut y = oy - grid_spacing;
        while y > 0.0 { draw_line(0.0, y, screen_width(), y, 1.0, grid_color); y -= grid_spacing; }

        // --- 绘制坐标轴 ---
        draw_line(0.0, oy, screen_width(), oy, 1.0, RED); // X 轴
        draw_line(ox, 0.0, ox, screen_height(), 1.0, GREEN); // Y 轴

        draw_text("X (m)", screen_width() - 50.0, oy - 10.0, 20.0, RED);
        draw_text("Y (m)", ox + 10.0, 30.0, 20.0, GREEN);
    }

    /// 绘制右侧的信息面板
    fn draw_info_panel(&self) {
        let info_x = screen_width() - 300.0;
        let info_y = 20.0;
        let line_height = 25.0;

        if !self.trajectory.is_empty() {
            draw_text(
                &format!("F: {}/{}", self.current_frame + 1, self.trajectory.len()),
                info_x, info_y, 20.0, WHITE,
            );

            // 确保我们不会越界
            if self.current_frame < self.trajectory.len() {
                let state = &self.trajectory[self.current_frame];
                draw_text(
                    &format!("time: {:.3} s", self.current_frame as f64 * (1.0/60.0)), // 假设为 60fps
                    info_x, info_y + line_height * 1.0, 20.0, WHITE,
                );
                draw_text(
                    &format!("location (x, y): ({:.2}, {:.2}) m", state.position.x, state.position.y),
                    info_x, info_y + line_height * 2.0, 20.0, WHITE,
                );
                draw_text(
                    &format!("velocity (x, y): ({:.2}, {:.2}) m/s", state.velocity.x, state.velocity.y),
                    info_x, info_y + line_height * 3.0, 20.0, WHITE,
                );
                draw_text(
                    &format!("angle (x, y): ({:.1}, {:.1}) deg",
                             state.angle.x.to_degrees(), state.angle.y.to_degrees()),
                    info_x, info_y + line_height * 4.0, 20.0, WHITE,
                );
                draw_text(
                    &format!("angle velocity (x, y): ({:.1}, {:.1}) r/s",
                             state.angle_velocity.x, state.angle_velocity.y),
                    info_x, info_y + line_height * 5.0, 20.0, WHITE,
                );
            }
        }

        draw_text(
            "SPACE: Play/Pause | R: Reset",
            20.0, screen_height() - 30.0, 20.0, GRAY,
        );
        draw_text(
            "rool: scaling | Left mouse button drag: Pan",
            250.0, screen_height() - 30.0, 20.0, GRAY,
        );
    }

    // --- 坐标 & 控制 ---

    /// 坐标转换: (X, Y) 物理世界 -> (X_px, Y_px) 屏幕
    fn world_to_screen(&self, world_pos: Vector2D) -> Vec2 {
        vec2(
            self.world_origin_on_screen.x + (world_pos.x * self.scale) as f32,
            self.world_origin_on_screen.y - (world_pos.y * self.scale) as f32, // Y 轴反转
        )
    }

    /// (私有) 坐标转换: (X_px, Y_px) 屏幕 -> (X, Y) 物理世界
    fn screen_to_world(&self, screen_pos: Vec2) -> Vector2D {
        Vector2D {
            x: (screen_pos.x - self.world_origin_on_screen.x) as f64 / self.scale,
            y: (self.world_origin_on_screen.y - screen_pos.y) as f64 / self.scale, // Y 轴反转
        }
    }

    /// (私有) 处理视图平移和缩放
    fn handle_view_controls(&mut self) {
        // --- 缩放 (鼠标滚轮) ---
        let scroll = mouse_wheel().1;
        if scroll.abs() > 0.1 {
            let mouse_pos_screen = mouse_position().into();
            let mouse_pos_world_before = self.screen_to_world(mouse_pos_screen);

            // 缩放
            let zoom_factor = 3.0;
            if scroll > 0.0 {
                self.scale *= zoom_factor;
            } else {
                self.scale /= zoom_factor;
            }

            // (让缩放以鼠标为中心)
            let mouse_pos_world_after = self.screen_to_world(mouse_pos_screen);
            let world_delta = mouse_pos_world_before - mouse_pos_world_after;

            self.world_origin_on_screen.x += (world_delta.x * self.scale) as f32;
            self.world_origin_on_screen.y -= (world_delta.y * self.scale) as f32; // Y 轴反转
        }

        // --- 平移 (鼠标中键) ---
        let mouse_pos = mouse_position().into();
        if is_mouse_button_pressed(MouseButton::Left) {
            self.is_panning = true;
            self.last_mouse_pos = mouse_pos;
        }
        if is_mouse_button_released(MouseButton::Left) {
            self.is_panning = false;
        }
        if self.is_panning {
            let delta = mouse_pos - self.last_mouse_pos;
            self.world_origin_on_screen += delta;
            self.last_mouse_pos = mouse_pos;
        }
    }

    // --- 公共控制 API ---

    /// (主循环调用) 检查并执行用户输入
    pub fn check_input(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.toggle_play();
        }
        if is_key_pressed(KeyCode::R) {
            self.reset();
        }
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.is_playing = true; // 重置后自动播放
    }

    pub fn trajectory_len(&self) -> usize {
        self.trajectory.len()
    }

    pub fn has_trajectory(&self) -> bool {
        !self.trajectory.is_empty()
    }
}