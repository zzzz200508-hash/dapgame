use crate::basic_structs::Vector2D;
use crate::physics::parameters::*; 
//use crate::basic_structs::Vector2D;
use crate::solver2::{OdeSystem, VectorSpace};
use crate::physics::simulation::*; 
use crate::physics::simulation::{polygon_area, clip_polygon_below_line};


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
//微分方程
impl CustomSettings{
    pub fn deriv_flying(&self, _t: f64, stone: &StoneInfo) -> StoneInfo{
        StoneInfo {
            position: stone.velocity, 
            velocity: Vector2D {
                x: 0.0,   // 水平速度恒定
                y: -self.gravity, // 垂直自由落体
            }, 
            angle: stone.angle_velocity, 
            angle_velocity: Vector2D { x: (0.0), y: (0.0) }, // TODO:

        }
    }
    pub fn deriv_bouncing(&self, _t: f64, stone: &StoneInfo) -> StoneInfo{
        StoneInfo {
            position: stone.velocity, 
            velocity: self.compute_force(stone) * (1.0 / self.M),
            angle: stone.angle_velocity, 
            angle_velocity: self.compute_angular_acceleration(stone),
        }
    }
    pub fn deriv_sinking(&self, _t:f64, stone: &StoneInfo) -> StoneInfo{
        StoneInfo {
            position: Vector2D { x: 0.0, y: 0.0 },
            velocity: Vector2D { x: 0.0, y: 0.0 },
            angle: Vector2D{x: 0.0, y:0.0}, // 角度保持最后状态
            angle_velocity: Vector2D { x: 0.0, y: 0.0 }, // 停止旋转
        }
    }

}
//水动力
impl CustomSettings{
    pub fn compute_force(&self, stone: &StoneInfo) -> Vector2D {
        let velocity = stone.velocity;
        let speed_sq = velocity.length_squared();

        // 重力 (向下为负)
        let f_gravity = Vector2D { x: 0.0, y: -self.M * self.gravity };

        // 如果速度极小或没有浸没，只受重力
        if speed_sq < 1e-6 || self.Sim <= 1e-9 {
            return f_gravity;
        }

        let speed = speed_sq.sqrt();
        // 速度单位向量 (运动方向)
        let dir_v = velocity * (1.0 / speed);

        // --- 1. 阻力 (Drag) ---
        // 必须严格与速度方向相反！这保证了能量总是被消耗。
        // F_drag = -0.5 * rho * S * Cf * v^2 * unit_v
        let f_drag_mag = 0.5 * self.rho * self.Sim * self.Cf * speed_sq;
        let f_drag = dir_v * -f_drag_mag;

        // --- 2. 升力 (Lift) ---
        // 必须垂直于速度方向。这保证升力不做功 (不会凭空增加能量)。
        // 将速度向量旋转 90 度: (x, y) -> (-y, x)
        let mut dir_lift = Vector2D { x: -dir_v.y, y: dir_v.x };

        // 确保升力总是指向上方 (抵抗重力)
        if dir_lift.y < 0.0 { dir_lift = dir_lift * -1.0; }

        // 升力系数修正：通常攻角(pitch)越大，升力越大（简单近似）
        // 这里为了稳定性，我们保持基础 Cl，但添加出水阻尼
        let mut f_lift_mag = 0.5 * self.rho * self.Sim * self.Cl * speed_sq;

        // [重要] 出水阻尼 (Exit Damping)
        // 如果石头正在向上运动 (vy > 0)，流体分离会导致升力急剧下降。
        // 如果不加这个，石头会在出水瞬间被巨大的力“弹”飞到高空。
        if velocity.y > 0.0 {
            f_lift_mag *= 0.3; // 向上运动时，升力大幅衰减
        }

        let f_lift = dir_lift * f_lift_mag;

        f_drag + f_lift + f_gravity
    }
}


impl CustomSettings {
    pub fn compute_angular_acceleration(&self, stone: &StoneInfo) -> Vector2D {
        // y 分量：自转角速度衰减
        let angular_spin = -self.beta * stone.angle_velocity.y;

        // 无浸没面积则无力矩
        if self.Sim <= 0.0 {
            return Vector2D { x: 0.0, y: angular_spin };
        }

        // 水下部分
        let clipped = self.current_submerged_polygon.clone();
        let force_point = pressure_center(&clipped);

        // 力臂
        let r = force_point - stone.position;

        // 水动力
        let F = self.compute_force(stone);

        // torque = r × F
        let torque = r.x * F.y - r.y * F.x;
        let angular_tilt = torque / self.stone.inertia_tensor_x;

        Vector2D { x: angular_tilt, y: angular_spin }
    }

}






fn pressure_center(clipped: &Vec<Vector2D>) -> Vector2D {
    // 多边形质心
    let area = polygon_area(clipped);
    let mut cx = 0.0;
    let mut cy = 0.0;

    for i in 0..clipped.len() {
        let p1 = clipped[i];
        let p2 = clipped[(i + 1) % clipped.len()];
        let cross = p1.x * p2.y - p2.x * p1.y;
        cx += (p1.x + p2.x) * cross;
        cy += (p1.y + p2.y) * cross;
    }
    if area <= 0.001 { return Vector2D{ x: 0.0, y: 0.0 }}
    else {
        let factor = 1.0 / (6.0 * area);
        return Vector2D { x: cx * factor, y: cy * factor }
    }
}

