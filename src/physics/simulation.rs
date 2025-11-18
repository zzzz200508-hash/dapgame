use crate::physics::parameters::CustomSettings;
use crate::physics::parameters::Phase; 

use crate::basic_structs::Vector2D; 
use crate::solver2::RungeKuttaSolver; 


//颜子涵负责
//通过物理模型直接调用求解器进行求解（目前使用2.0版求解器，我在研究相关事宜）
//最后返回石头的状态向量
//Vec<StoneInfo>
// [0]:位置
// [1]:速度
// [2]:角度
// [3]:角速度
//Vec<StoneInfo, t>

#[derive(Clone)]
pub struct StoneInfo {
    pub position: Vector2D,
    pub velocity: Vector2D,
    pub angle: Vector2D,
    pub angle_velocity: Vector2D,
    
}

pub struct Stamp {
    pub t: f64, 
    pub state: StoneInfo, 
}

pub fn simulate(system: &mut CustomSettings,
                solver: &mut RungeKuttaSolver<StoneInfo>,
                dt: f64,
                max_steps: usize) -> Vec<Stamp> 
{
    let mut trajectory = Vec::new();

    for _ in 0..max_steps {

        // (1) 记录当前状态
        trajectory.push(
            Stamp{
                state: solver.state.clone(), 
                t: solver.t, 
            }
        );

        // (2) 更新浸水面积（必须在 phase 判断前）
        if system.phase == Phase::Bouncing {
            
            system.update_submerged_area(&solver.state);
        }
        // (3) 更新 phase
        system.update_phase(&solver.state);

        // 若已沉没，停止积分
        if system.phase == Phase::Sinking {
            println!("Phase=Sinking, simulation finished at t={}", solver.t);
            break;
        }

        // (4) 使用 RK4 进行一步积分
        solver.step(system, dt);
    }

    trajectory
}



impl CustomSettings {
    /// 根据当前 StoneInfo 更新 phase
    pub fn update_phase(&mut self, stone: &StoneInfo) {
        let _r = self.stone.d_max.sqrt();//石头最大半径

        match self.phase {
            Phase::Flying => {
                // 如果石头触碰到水面，切换到 Bouncing
                if stone.position.y -_r * stone.angle.x.sin() <= self.water_level {
                    self.phase = Phase::Bouncing;
                    println!("Phase switched: Flying -> Bouncing at y={}", stone.position.y);
                }
            }

            Phase::Bouncing => {
                // 是否离开水面
                if stone.position.y - _r * stone.angle.x.sin() > self.water_level && stone.velocity.y > 0.0 {
                    self.phase = Phase::Flying;
                    println!("Bouncing → Flying");
                    return;
                }
                // 判断是否应该沉入水底：
                // 可以用垂直力小于重力、速度太低等条件
                let vertical_force = self.compute_vertical_force(stone);
                let velocity_mag = stone.velocity.length(); // 你需要在 Vector2D/3D 里实现 length()

                if stone.position.y < -0.1 ||stone.velocity.x < 0.2{
                    self.phase = Phase::Sinking;
                    println!("Phase switched: Bouncing -> Sinking at y={}", stone.position.y);
                }
            }

            Phase::Sinking => {
                // 已经沉入，不需要做任何事
            }
        }
    }

    /// 可选：计算垂直方向的受力
    fn compute_vertical_force(&self, stone: &StoneInfo) -> f64 {
        // 简化：重力 + 升力 + 阻力
        // Fx = 0.5 * rho * v^2 * area * Cl 之类
        let v2 = stone.velocity.length_squared();
        0.5 * self.rho * v2 * self.Sim * self.Cl - self.M * self.gravity
    }
}


impl CustomSettings {

    pub fn update_submerged_area(&mut self, stone_state: &StoneInfo) {
        let outline_world = self.outline_to_world(stone_state);

        let clipped = clip_polygon_below_line(&outline_world, self.water_level);

        // 保存下来（供 torque 使用）
        self.current_submerged_polygon = clipped.clone();

        if clipped.len() < 3 {
            self.Sim = 0.0;
        } else {
            self.Sim = polygon_area(&clipped);
        }
    }


    pub fn outline_to_world(&self, stone: &StoneInfo) -> Vec<Vector2D> {
        // 1. 自转角 (Spin / angle.y)
        // 决定石头在该时刻呈现的“形状”姿态 (在自身坐标系内旋转)
        let spin = stone.angle.y;
        let cos_spin = spin.cos();
        let sin_spin = spin.sin();

        // 2. 俯仰角 (Pitch / angle.x)
        // 决定石头整体在世界坐标系中的倾角
        let pitch = stone.angle.x;
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();

        self.stone.outline_com.iter()
            .map(|p| {
                // A. 先进行自转 (Local Rotation)
                // 绕石片中心 (0,0) 旋转
                let x_spun = p.x * cos_spin - p.y * sin_spin;
                let y_spun = p.x * sin_spin + p.y * cos_spin;

                // B. 再进行俯仰 (World Rotation) 并平移
                // 将自转后的点，应用俯仰角旋转，然后加上质心位置
                Vector2D {
                    x: stone.position.x + (x_spun * cos_pitch - y_spun * sin_pitch),
                    y: stone.position.y + (x_spun * sin_pitch + y_spun * cos_pitch),
                }
            })
            .collect()
    }
}

pub fn clip_polygon_below_line(poly: &[Vector2D], line_y: f64) -> Vec<Vector2D> {
    let mut output = Vec::new();
    let n = poly.len();

    if n == 0 {
        return output;
    }

    for i in 0..n {
        let cur = poly[i];
        let next = poly[(i + 1) % n];

        let cur_inside = cur.y < line_y;
        let next_inside = next.y < line_y;

        match (cur_inside, next_inside) {
            // Both inside → keep next
            (true, true) => {
                output.push(next);
            }

            // cur inside → next outside
            // keep intersection only
            (true, false) => {
                if let Some(inter) = intersect_with_horizontal(cur, next, line_y) {
                    output.push(inter);
                }
            }

            // cur outside → next inside
            // add intersection + next
            (false, true) => {
                if let Some(inter) = intersect_with_horizontal(cur, next, line_y) {
                    output.push(inter);
                }
                output.push(next);
            }

            // both outside → add nothing
            (false, false) => {}
        }
    }

    output
}

fn intersect_with_horizontal(p1: Vector2D, p2: Vector2D, y: f64) -> Option<Vector2D> {
    // Line segment p1→p2 intersects horizontal line y?
    if (p1.y - y) * (p2.y - y) > 0.0 {
        return None; // same side, no intersection
    }
    if (p1.y - p2.y).abs() < 1e-12 {
        return None; // horizontal segment
    }

    let t = (y - p1.y) / (p2.y - p1.y);

    Some(Vector2D {
        x: p1.x + t * (p2.x - p1.x),
        y,
    })
}

pub fn polygon_area(poly: &[Vector2D]) -> f64 {
    if poly.len() < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    let n = poly.len();

    for i in 0..n {
        let j = (i + 1) % n;
        area += poly[i].x * poly[j].y - poly[j].x * poly[i].y;
    }

    area.abs() * 0.5
}
