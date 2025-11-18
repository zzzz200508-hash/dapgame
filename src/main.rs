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

// [修正] 根据提供的文件结构引入模块
use crate::physics::parameters::{CustomSettings, Phase};
use crate::physics::simulation::StoneInfo;

#[macroquad::main("2D 水漂模拟 (Skipping Stone Simulation)")]
async fn main() {
    // [新增] 外层循环，用于支持 Restart 功能
    loop {
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

            // 1. 计算石片物理属性
            let stone_props = StoneProperties::new(&blueprint);

            if stone_props.mass <= 1e-9 {
                println!("错误: 石片质量无效，无法模拟");
                return;
            }

            // 2. 初始化物理环境
            let mut system = CustomSettings::new(9.81, stone_props.clone());

            // 3. 初始化渲染器
            let mut renderer = SimulationRenderer::new(stone_props, 8000.0);

            // 4. 初始化求解器
            let mut solver = RungeKuttaSolver::new(0.0, y0);

            // 记录初始帧
            renderer.add_state(solver.state.clone());

            let simulation_dt: f64 = 0.001;
            let steps_per_frame: usize = 1;

            // --- 游戏/评分状态变量 ---
            let mut skip_count = 0;          // 水漂次数
            let mut score_air_time = 0.0;    // 得分 (有效滞空时间)
            let mut has_touched_water = false; // 是否已经开始接触水面 (用于开始计分)
            let mut is_game_over = false;    // 游戏结束标志 (沉没)

            // --- 阶段 3: 主循环 ---
            let mut restart_requested = false;
            loop {

                // 1. 物理计算子步 (仅当游戏未结束时进行)
                if !is_game_over {
                    for _ in 0..steps_per_frame {

                        // [评分逻辑] 记录这一步之前的 Y 坐标
                        let y_prev = solver.state.position.y;

                        // (A) 更新浸没面积
                        if system.phase == Phase::Bouncing {
                            system.update_submerged_area(&solver.state);
                        }

                        // (B) 更新相位
                        system.update_phase(&solver.state);

                        // (C) 检查是否沉没 (游戏结束)
                        if system.phase == Phase::Sinking {
                            is_game_over = true;
                            println!("Game Over! Final Score: {:.3}s, Skips: {}", score_air_time, skip_count);
                            break; // 停止物理步进
                        }

                        // (D) 执行一步积分 (更新 solver.state)
                        solver.step(&system, simulation_dt);

                        // [评分逻辑] 记录这一步之后的 Y 坐标
                        let y_curr = solver.state.position.y;

                        // 1. 检测首次入水
                        if !has_touched_water && y_curr <= 0.0 {
                            has_touched_water = true;
                        }

                        // 2. 如果已经入过水，开始处理计分
                        if has_touched_water {
                            // 检测水漂：上一步在水下 (或刚好在水面)，这一步在水上
                            if y_prev <= 0.0 && y_curr > 0.0 {
                                skip_count += 1;
                            }

                            // 累加滞空时间 (作为分数)
                            if y_curr > 0.0 {
                                score_air_time += simulation_dt;
                            }
                        }
                    }
                }

                // 2. 渲染与交互
                // 即使游戏结束，也可以继续绘制轨迹和操作视角，只是不再添加新状态
                if !is_game_over {
                    renderer.add_state(solver.state.clone());
                }
                renderer.check_input();
                renderer.draw_and_update();

                // 3. 绘制 UI (分数与游戏状态)
                draw_game_ui(skip_count, score_air_time, is_game_over);

                // 4. 检查重启
                if renderer.should_restart {
                    restart_requested = true;
                    break;
                }

                next_frame().await
            }

            if restart_requested {
                continue;
            }

        } else {
            println!("编辑器已退出，未开始模拟。");
            break;
        }
    }
}

// 辅助函数：绘制游戏UI
fn draw_game_ui(skip_count: i32, score_time: f64, is_game_over: bool) {
    let font_size = 30.0;
    let padding = 20.0;

    // 左上角实时数据
    draw_text(&format!("Skips: {}", skip_count), padding, 40.0, font_size, WHITE);
    draw_text(&format!("Score: {:.3}s", score_time), padding, 75.0, font_size, WHITE);

    // 游戏结束画面
    if is_game_over {
        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;

        // 半透明背景板
        let panel_w = 400.0;
        let panel_h = 250.0;
        draw_rectangle(
            center_x - panel_w/2.0,
            center_y - panel_h/2.0,
            panel_w, panel_h,
            Color::from_rgba(0, 0, 0, 200)
        );
        draw_rectangle_lines(
            center_x - panel_w/2.0,
            center_y - panel_h/2.0,
            panel_w, panel_h,
            3.0, RED
        );

        // 文字
        let title = "GAME OVER";
        let title_dims = measure_text(title, None, 50, 1.0);
        draw_text(title, center_x - title_dims.width/2.0, center_y - 50.0, 50.0, RED);

        let score_text = format!("Final Score: {:.3}s", score_time);
        let score_dims = measure_text(&score_text, None, 30, 1.0);
        draw_text(&score_text, center_x - score_dims.width/2.0, center_y + 10.0, 30.0, WHITE);

        let skip_text = format!("Total Skips: {}", skip_count);
        let skip_dims = measure_text(&skip_text, None, 30, 1.0);
        draw_text(&skip_text, center_x - skip_dims.width/2.0, center_y + 50.0, 30.0, WHITE);

        let hint = "Press 'Restart' to try again";
        let hint_dims = measure_text(hint, None, 20, 1.0);
        draw_text(hint, center_x - hint_dims.width/2.0, center_y + 100.0, 20.0, GRAY);
    }
}