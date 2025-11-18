//备注：角度、角速度.x均为与水平面角度,
//另外一个是自传.

mod bezier;
mod basic_structs;
mod print;
mod solver2;
mod stone_editor;
mod stone_phy;
mod physics;

use crate::physics::parameters::*;
use crate::physics::simulation::StoneInfo;
use macroquad::prelude::*;
use crate::stone_editor::{StoneBlueprint, StoneEditor};
use crate::basic_structs::*;
use crate::print::*;
use crate::solver2::{OdeSystem, RungeKuttaSolver, VectorSpace};
use crate::stone_phy::StoneProperties;

/// 注意： `#[macroquad::main]` 宏将此函数转换为 `async`
/// 并为 Macroquad 窗口提供一个主循环。
#[macroquad::main("2D 水漂模拟 (Skipping Stone Simulation)")]
async fn main() {

    // --- 阶段 1: 参数获取 (通过 StoneEditor) ---
    // 1. 初始化编辑器
    let mut editor = StoneEditor::new();
    // 2. 运行编辑器循环
    // `editor.run().await` 会阻塞执行，直到用户
    // 在 UI 中点击 "START SIMULATION"
    println!("正在启动参数编辑器...");
    editor.run().await;

    // 3. 从编辑器结果中提取参数
    // `editor.result` 是 `Option<(StoneBlueprint, StoneInfo)>`
    if let Some((blueprint, y0)) = editor.result {
        println!("\n--- 参数获取成功 ---");

        // --- 参数 1: StoneBlueprint (石片蓝图) ---
        // (来自 "FINISH & BUILD" 和 "CONFIRM" 步骤)
        let stone_shape_points: &Vec<Vector2D> = &blueprint.points;
        let stone_thickness: f64 = blueprint.thickness; // (单位: 米)

        println!("  [参数 1: 蓝图]");
        println!("    > 石片名称: {}", blueprint.name);
        println!("    > 石片厚度: {:.3} m", stone_thickness);
        println!("    > 石片轮廓点数: {}", stone_shape_points.len());

        // --- 参数 2: StoneInfo (石片初始状态, y0) ---
        // (来自 "SET INITIAL CONDITIONS" 步骤)
        // y0 本身就是完整的初始状态

        println!("  [参数 2: 初始状态 (y0)]");
        println!("    > 初始位置 (x, y):    ({:.2}, {:.2}) m", y0.position.x, y0.position.y);
        println!("    > 初始速度 (x, y):    ({:.2}, {:.2}) m/s", y0.velocity.x, y0.velocity.y);
        println!("    > 初始角度:           {:.2} rad ({:?} deg)", y0.angle.x, y0.angle.y.to_degrees());
        println!("    > 初始角速度:         {:.2} rad/s", y0.angle_velocity.x);

        println!("----------------------------\n");
        println!("正在启动物理模拟...");

        // --- 阶段 2: 模拟执行 ---

        let simulation_dt: f64 = 0.001; // [要求 1] 模拟步长
        let steps_per_frame: usize = 10;  // [要求 2] 每帧渲染 10 步

        // 2. [新增] 初始化物理系统 (CustomSettings)
        // (这里使用基于 derivative.rs 的合理默认值)
        let mut system = CustomSettings {
            gravity: -9.81,     // 重力 (Y 轴负方向)
            rho: 1000.0,        // 水的密度
            beta: 0.1,          // 阻尼系数 (假设)
            Cl: 1.0,            // 升力系数 (假设)
            Cf: 0.1,            // 阻力系数 (假设)
            M: 0.0,             // 附加质量 (假设)
            Sim: 0.0,           // 浸没面积 (初始为0)
            current_submerged_polygon: vec![], // 浸没多边形 (初始为空)
            phase: Phase::Flying, // 初始状态为飞行
        };

        // 3. [新增] 初始化 2D 渲染器
        // (stone_editor.rs 在 world_to_screen 中使用了 8000.0)
        let mut renderer = SimulationRenderer::new(stone_props, 8000.0);

        // 4. 初始化求解器
        let mut solver = RungeKuttaSolver::new(0.0, y0);

        // 5. [新增] 将初始状态添加到渲染器
        renderer.add_state(solver.state);

        // --- 阶段 3: 游戏/渲染循环 ---
        loop {
            // --- 1. 逻辑更新 (Fixed Timestep) ---

            // [要求 2] 每帧渲染执行 10 次物理步进
            for _ in 0..steps_per_frame {

                // 1a. (关键) 更新系统状态
                // (这会检查碰撞, 切换 Phase, 计算浸没面积 Sim 等)
                update_system_state(&mut system, &solver.state);

                // 1b. 运行一步物理计算
                solver.step(&system, simulation_dt);
            }

            // --- 2. 渲染 ---

            // 2a. 将最新的状态添加到轨迹中
            renderer.add_state(solver.state);

            // 2b. 处理渲染器输入 (Pause/Play/Zoom/Pan)
            renderer.check_input();

            // 2c. 绘制所有内容 (轨迹, 石头, UI)
            renderer.draw_and_update(); // 这会绘制主窗口和小窗

            // 2d. 等待 Macroquad 准备下一帧
            next_frame().await
        }

    } else {
        // (如果用户关闭了编辑器窗口而没有点击 "START SIMULATION")
        println!("编辑器已退出，未开始模拟。");
    }
}