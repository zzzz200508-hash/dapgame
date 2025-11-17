mod bezier;
mod calculate;
mod basic_structs;
mod print;
mod solver2;
mod stone_editor;

use macroquad::prelude::*;
use crate::stone_editor::StoneEditor;
use crate::calculate::*;
use crate::basic_structs::*;
use crate::print::*;
use crate::solver2::{OdeSystem, RungeKuttaSolver, VectorSpace};

// 注意：使用 Macroquad 时，main 函数需要特殊标记
#[macroquad::main("Skipping Stone Simulation")]
async fn main() {
    // 1. 初始化编辑器
    let mut editor = StoneEditor::new();

    // 2. 运行编辑器循环 (这会阻塞直到用户点击 Finish)
    println!("Starting Editor...");
    editor.run().await;

    // 3. 获取结果
    if let Some(blueprint) = editor.result {
        println!("Stone Created!");
        println!("Name: {}", blueprint.name);
        println!("Thickness: {}", blueprint.thickness);
        println!("Point Count: {}", blueprint.points.len());

        // TODO: 下一步：将 blueprint 转换为 3D Mesh 和 Tensor3d (StoneFactory)
        // let stone_3d = StoneFactory::create_mesh(blueprint);
        // start_simulation(stone_3d);
    } else {
        println!("Exited without creating a stone.");
    }

    // 这里的 loop 是为了防止窗口直接关闭，后续将替换为 3D 模拟循环
    loop {
        clear_background(LIGHTGRAY);
        draw_text("Stone Ready. Simulation Pending...", 20.0, 40.0, 30.0, BLACK);
        next_frame().await
    }
    
    
    
    //后面是待增加部分，暂时采用ai示例。
    let dt = 0.01; // 物理模拟通常需要更小的时间步长

    // 1. 定义物理系统（例如：地球重力）
    let system = CustomSettings::new(9.81);

    // 2. 定义石头的初始状态
    let y0 = StoneInfo {
        position: Vector2D::new(0.0, 0.0),
        velocity: Vector2D::new(0.0, 0.0),
        angle: Vector2D::new(0.0, 0.0),
        angle_velocity: Vector2D::new(0.0, 0.0),
    };

    // 3. 初始化求解器
    let mut solver = RungeKuttaSolver::new(0.0, y0);

    // 4. 运行模拟循环
    for i in 0..1000 {
        solver.step(&system, dt);

        if i % 100 == 0 {
            println!("Time: {:.2}, Position: {:?}", solver.t, solver.state.position);
        }
    }
}
