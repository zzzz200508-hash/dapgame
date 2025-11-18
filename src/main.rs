//备注：角度、角速度.x均为与水平面角度,
//另外一个是自传.

mod bezier;
mod basic_structs;
mod print;
mod solver2;
mod stone_editor;
mod stone_phy;
// 引用 physics 模块 (对应 src/physics/mod.rs)
mod physics;

use macroquad::prelude::*;
use crate::stone_editor::StoneEditor;
use crate::print::SimulationRenderer;
use crate::solver2::RungeKuttaSolver;
use crate::stone_phy::StoneProperties;

use crate::physics::parameters::{CustomSettings, Phase};

#[macroquad::main("2D 水漂模拟 (Skipping Stone Simulation)")]
async fn main() {

    // --- 阶段 1: 参数获取 (通过 StoneEditor) ---
    let mut editor = StoneEditor::new();

    println!("正在启动参数编辑器...");
    editor.run().await;

    // 获取编辑器结果
    if let Some((blueprint, y0)) = editor.result {
        println!("\n--- 参数获取成功 ---");
        println!("  > 石片名称: {}", blueprint.name);
        println!("  > 初始状态 Pos: ({:.2}, {:.2})", y0.position.x, y0.position.y);

        println!("正在启动物理模拟...");

        // --- 阶段 2: 初始化 ---

        // 1. 计算石片物理属性 (Mass, Inertia, Collision Mesh)
        // 这一步是核心，必须先将蓝图转换为物理属性
        let stone_props = StoneProperties::new(&blueprint);

        if stone_props.mass <= 1e-9 {
            println!("错误: 石片质量无效，无法模拟");
            return;
        }

        // 2. 初始化物理环境 (CustomSettings)
        // [修正] 使用 parameters.rs 中提供的 new 构造函数
        // 传入重力 -9.81 和 计算好的 stone_props
        let mut system = CustomSettings::new(9.81, stone_props.clone());

        // 3. 初始化渲染器
        // 缩放比例 8000.0 (像素/米)
        let mut renderer = SimulationRenderer::new(stone_props, 8000.0);

        // 4. 初始化求解器 (RK4)
        let mut solver = RungeKuttaSolver::new(0.0, y0);

        // 记录初始帧
        renderer.add_state(solver.state.clone());

        let simulation_dt: f64 = 0.001;
        let steps_per_frame: usize = 10;

        // --- 阶段 3: 主循环 ---
        loop {
            // 1. 物理计算子步 (Fixed Timestep)
            for _ in 0..steps_per_frame {

                // [关键逻辑] 复刻 simulation.rs 中的 simulate 函数逻辑

                // (A) 更新浸没面积 (仅在接触水面 Bouncing 时计算，优化性能)
                // 依据 simulation.rs 的逻辑：if system.phase == Phase::Bouncing { ... }
                if system.phase == Phase::Bouncing {
                    system.update_submerged_area(&solver.state);
                }

                // (B) 更新相位 (检测 Flying <-> Bouncing <-> Sinking)
                system.update_phase(&solver.state);

                // (C) 执行一步积分
                // 如果是 Sinking 状态，derivative.rs 中的实现会让速度归零，依然可以安全 step
                solver.step(&system, simulation_dt);
            }

            // 2. 渲染与交互
            renderer.add_state(solver.state.clone());
            renderer.check_input();
            renderer.draw_and_update();

            if system.phase == Phase::Sinking {
                println!("Phase=Sinking, simulation finished at t={}", solver.t);
                
            }

            next_frame().await
        }

    } else {
        println!("编辑器已退出，未开始模拟。");
    }
}