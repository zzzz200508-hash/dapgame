use crate::basic_structs::Vector2D;
use crate::physics::parameters::*;
use crate::solver2::{OdeSystem, VectorSpace};
use crate::physics::simulation::*;
use crate::physics::simulation::polygon_area;

impl VectorSpace for StoneInfo {
    fn add(&self, other: &Self) -> Self {
        Self {
            position: self.position + other.position,
            velocity: self.velocity + other.velocity,
            angle: self.angle + other.angle,
            angle_velocity: self.angle_velocity + other.angle_velocity,
        }
    }

    fn scale(&self, scalar: f64) -> Self {
        Self {
            position: self.position * scalar,
            velocity: self.velocity * scalar,
            angle: self.angle * scalar,
            angle_velocity: self.angle_velocity * scalar,
        }
    }
}

impl OdeSystem<StoneInfo> for CustomSettings {
    fn derivatives(&self, _t: f64, stone: &StoneInfo) -> StoneInfo {
        match self.phase {
            Phase::Flying => self.deriv_flying(_t, stone),
            Phase::Bouncing => self.deriv_bouncing(_t, stone),
            Phase::Sinking => self.deriv_sinking(_t, stone),
        }
    }
}

impl CustomSettings {
    pub fn deriv_flying(&self, _t: f64, stone: &StoneInfo) -> StoneInfo {
        StoneInfo {
            position: stone.velocity,
            velocity: Vector2D {
                x: 0.0,
                y: - self.gravity,
            },
            angle: stone.angle_velocity,
            angle_velocity: Vector2D { x: 0.0, y: 0.0 },
        }
    }

    pub fn deriv_bouncing(&self, _t: f64, stone: &StoneInfo) -> StoneInfo {
        // [核心修复] 实时计算当前 RK4 子步的浸没状态
        // 不再依赖 self.current_submerged_polygon (它是上一帧的缓存)
        let (sim, clipped) = self.calculate_instant_submerged(stone);

        // 1. 计算水动力 (不含重力)
        let f_hydro = self.compute_hydro_force(stone, sim);

        // 2. 计算总合力 (水动力 + 重力) -> 用于线加速度
        let f_gravity = Vector2D { x: 0.0, y: self.M * self.gravity };
        let f_total = f_hydro + f_gravity;

        // 3. 计算线加速度 F=ma
        let mass = if self.M > 1e-9 { self.M } else { 1.0 };
        let acceleration = f_total * (1.0 / mass);

        // 4. 计算角加速度 (力矩)
        // 传入 f_hydro，因为只有水动力产生相对于质心的力矩
        let angular_acc = self.compute_angular_acceleration(stone, sim, &clipped, f_hydro);

        StoneInfo {
            position: stone.velocity,
            velocity: acceleration,
            angle: stone.angle_velocity,
            angle_velocity: angular_acc,
        }
    }

    pub fn deriv_sinking(&self, _t:f64, stone: &StoneInfo) -> StoneInfo {
        StoneInfo {
            position: Vector2D { x: 0.0, y: 0.0 },
            velocity: Vector2D { x: 0.0, y: 0.0 },
            angle: Vector2D { x: 0.0, y: 0.0 },
            angle_velocity: Vector2D { x: 0.0, y: 0.0 },
        }
    }
}

impl CustomSettings {
    // [新增] 辅助函数：根据传入的 StoneInfo 实时计算浸没多边形
    // 确保了 clipped 变量是有计算来源的
    fn calculate_instant_submerged(&self, stone: &StoneInfo) -> (f64, Vec<Vector2D>) {
        // 调用 simulation.rs 中的逻辑
        let outline_world = self.outline_to_world(stone);
        let clipped = clip_polygon_below_line(&outline_world, self.water_level);

        let sim = if clipped.len() < 3 {
            0.0
        } else {
            polygon_area(&clipped)
        };
        (sim, clipped)
    }

    pub fn compute_hydro_force(&self, stone: &StoneInfo, sim: f64) -> Vector2D {
        let velocity = stone.velocity;
        let speed_sq = velocity.length_squared();

        // 如果没有接触水，或者速度极小，没有水动力
        if sim <= 1e-9 {
            return Vector2D { x: 0.0, y: 0.0 };
        }

        let speed = speed_sq.sqrt();
        // 避免除以零
        let dir_v = if speed > 1e-6 { velocity * (1.0 / speed) } else { Vector2D { x: 0.0, y: 0.0 } };

        // --- 1. 基础阻力 (Form Drag) ---
        // 与速度方向相反
        let f_drag_mag = 0.5 * self.rho * sim * self.Cf * speed_sq;
        let f_drag = dir_v * -f_drag_mag;

        // --- 2. 基础升力 (Lift) ---
        // 垂直于速度方向
        let mut dir_lift = Vector2D { x: -dir_v.y, y: dir_v.x };
        if dir_lift.y < 0.0 { dir_lift = dir_lift * -1.0; } // 总是向上

        let f_lift_mag = 0.5 * self.rho * sim * self.Cl * speed_sq;
        let f_lift = dir_lift * f_lift_mag;

        // --- 3. [关键] 垂直混合阻尼 (Hybrid Vertical Damping) ---
        // 为了解决“上下震荡”，我们需要强力的垂直阻尼。
        // 混合了 平方项(高速) 和 线性项(低速)。

        let damping_quad = 20.0; // 高速阻尼系数 (猛烈撞击时生效)
        let damping_lin = 10.0;   // 低速阻尼系数 (稳定水面浮动)

        let vy = velocity.y;
        // F_damp = -rho * Area * ( k1 * v^2 + k2 * v )
        // 注意符号：阻尼力总是反向于 vy
        let damp_mag = 0.5 * self.rho * sim * (damping_quad * vy.abs() + damping_lin);
        let f_vertical_damp_y = -damp_mag * vy;

        let f_vertical_damp = Vector2D { x: 0.0, y: f_vertical_damp_y };

        f_drag + f_lift + f_vertical_damp
    }

    // [重构] 角加速度计算：增强稳定性
    pub fn compute_angular_acceleration(&self, stone: &StoneInfo, sim: f64, clipped: &Vec<Vector2D>, f_hydro: Vector2D) -> Vector2D {
        // 1. 自转阻尼 (Spin Damping)
        // 这是一个纯耗散项
        let spin_damping = -self.beta * stone.angle_velocity.y;

        if sim <= 1e-9 {
            return Vector2D { x: 0.0, y: spin_damping };
        }

        // 2. 计算压力中心 (Center of Pressure)
        let force_point = pressure_center(clipped);

        // 力臂 r = 压力中心 - 质心
        let mut r = force_point - stone.position;

        // [安全修正] 限制力臂长度
        // 如果数值计算导致压力中心偏离太远，强制拉回，防止力矩爆炸
        let max_arm = 0.2; // 假设石头半径大概在这个范围
        if r.length_squared() > max_arm * max_arm {
            r = r.normalize() * max_arm;
        }

        // 3. 水动力力矩 Torque = r x F_hydro
        let torque = r.x * f_hydro.y - r.y * f_hydro.x;

        // 4. [关键] 俯仰阻尼 (Pitch Damping)
        // 水对石片翻转有巨大的抵抗力 (Added Mass Inertia / Viscosity)
        // 系数需要足够大以抑制“点头”震荡
        let pitch_damping_coeff = 5.0;
        // 阻尼力矩与 浸没面积 和 角速度 成正比
        let pitch_damping_torque = -0.5 * self.rho * sim * pitch_damping_coeff * stone.angle_velocity.x;

        let total_torque_x = torque + pitch_damping_torque;

        let inertia = if self.stone.inertia_tensor_x > 1e-9 { self.stone.inertia_tensor_x } else { 0.1 };

        // 5. [安全修正] 限制最大角加速度
        let pitch_acc = total_torque_x / inertia;
        let max_acc = 500.0;
        let pitch_acc_clamped = pitch_acc.clamp(-max_acc, max_acc);

        Vector2D { x: pitch_acc_clamped, y: spin_damping }
    }
}

// 压力中心计算
fn pressure_center(clipped: &Vec<Vector2D>) -> Vector2D {
    if clipped.len() < 3 { return Vector2D::new(0.0, 0.0); }

    let area = polygon_area(clipped);
    // 防止面积过小导致除以零
    if area.abs() < 1e-9 {
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        for p in clipped {
            sum_x += p.x;
            sum_y += p.y;
        }
        let n = clipped.len() as f64;
        return Vector2D { x: sum_x / n, y: sum_y / n };
    }

    let mut cx = 0.0;
    let mut cy = 0.0;

    for i in 0..clipped.len() {
        let p1 = clipped[i];
        let p2 = clipped[(i + 1) % clipped.len()];
        let cross = p1.x * p2.y - p2.x * p1.y;
        cx += (p1.x + p2.x) * cross;
        cy += (p1.y + p2.y) * cross;
    }

    let factor = 1.0 / (6.0 * area);
    Vector2D { x: cx * factor, y: cy * factor }
}