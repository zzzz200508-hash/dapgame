//备注：角度、角速度.x均为与水平面角度,
//另外一个是自传.

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

        // 1. 定义模拟参数
        let dt: f64 = 0.001; // (推荐使用更小的时间步)

        // 2. 创建物理系统 (OdeSystem)
        // 注意: 你的 CustomSettings 应该被扩展
        // 以便接收 `blueprint` 来计算质量和转动惯量
        let system = CustomSettings {
            gravity: 9.81,
            // TODO: 在 CustomSettings 中添加石片属性
            // (例如: mass, inertia_tensor, area)
            // 这些都需要从 `blueprint` (shape + thickness) 计算得出
        };

        // 3. 初始化求解器
        // `y0` (参数 2) 被直接用于初始化求解器状态
        let mut solver = RungeKuttaSolver::new(0.0, y0);

        // --- 阶段 3: 游戏/渲染循环 ---
        // --- 逻辑更新 ---
        // A. (可选) 处理运行时输入 (例如暂停, 重启)
        // --- 渲染 ---
        clear_background(LIGHTGRAY); // 擦除上一帧
        // TODO: 在此处添加你的渲染逻辑
        // 绘制调试信息 (显示石片 X, Y 位置)
    } else {
        // (如果用户关闭了编辑器窗口而没有点击 "START SIMULATION")
        println!("编辑器已退出，未开始模拟。");
    }
}