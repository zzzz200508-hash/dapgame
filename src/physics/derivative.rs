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
                x: stone.velocity.x,   // 水平速度恒定
                y: self.gravity, // 垂直自由落体
            }, 
            angle: stone.angle_velocity, 
            angle_velocity: Vector2D { x: (0.0), y: (0.0) }, // TODO:

        }
    }
    pub fn deriv_bouncing(&self, _t: f64, stone: &StoneInfo) -> StoneInfo{
        StoneInfo {
            position: stone.velocity, 
            velocity: self.compute_force(stone), 
            angle: stone.angle_velocity, 
            angle_velocity: self.compute_angular_acceleration(stone),
        }
    }
    pub fn deriv_sinking(&self, _t:f64, stone: &StoneInfo) -> StoneInfo{
        StoneInfo {
            position: Vector2D { x: stone.position.x, y: stone.position.y },
            velocity: Vector2D { x: 0.0, y: 0.0 },
            angle: stone.angle, // 角度保持最后状态
            angle_velocity: Vector2D { x: 0.0, y: 0.0 }, // 停止旋转
        }
    }

}
//水动力
impl CustomSettings{
    pub fn compute_force(&self, stone: &StoneInfo) -> Vector2D {
        let v2 = stone.velocity.length_squared();
        let coeff = 0.5 * self.rho * v2 * self.Sim;

        // 力的 x 和 y 分量
        Vector2D {
            x: -coeff * (self.Cl * stone.angle.x.sin() + self.Cf * stone.angle.x.cos()),
            y: -self.M * self.gravity + coeff * (self.Cl * stone.angle.x.cos() - self.Cf * stone.angle.x.sin()),
        }
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

    let factor = 1.0 / (6.0 * area);
    Vector2D { x: cx * factor, y: cy * factor }
}

