打水漂物理模拟项目 (Skipping Stone Simulation)

项目简介

这是一个基于 Rust 和 Macroquad 引擎的 2D 物理模拟程序，旨在模拟石片在水面上“打水漂”的动力学过程。项目包含了石片几何编辑、物理属性计算、流体动力学受力分析以及实时可视化等功能。

核心模块与文件结构

1. 主程序入口

src/main.rs:

程序的入口点。

负责初始化流程：启动石片编辑器 -> 获取用户设计的石片参数 -> 初始化物理环境 (CustomSettings) 和求解器 (RungeKuttaSolver)。

包含主游戏循环：控制物理步进（Fixed Timestep）和渲染循环，协调物理计算与画面更新。

2. 石片编辑与几何处理

src/stone_editor.rs:

提供交互式的 GUI，允许用户通过贝塞尔曲线或手绘方式设计石片的截面形状。

允许设置初始投掷条件（位置、速度、攻角、自转角速度）。

输出 StoneBlueprint（几何蓝图）和 StoneInfo（初始状态）。

src/bezier.rs:

实现贝塞尔曲线算法，用于将用户输入的控制点转换为平滑的石片轮廓点。

src/stone_phy.rs:

负责将几何蓝图转换为物理属性。

计算石片的 质量 (Mass)、转动惯量 (Inertia)、质心 (Centroid)。

生成用于碰撞检测的 点云 (Collision Mesh)。

3. 物理引擎核心 (src/physics/ 模块)

这是项目的核心，负责模拟石片与流体的交互。

src/physics/mod.rs: 模块声明文件，组织 physics 子模块。

src/physics/parameters.rs:

定义物理环境参数结构体 CustomSettings（如重力、水密度、阻力系数）。

定义状态枚举 Phase（飞行 Flying、弹跳 Bouncing、沉没 Sinking）。

负责管理石片的实时物理属性。

src/physics/simulation.rs:

定义物理状态结构体 StoneInfo（位置、速度、角度、角速度）。

实现 碰撞检测与相位切换 (update_phase)：判断石片是否接触水面、是否沉没。

实现 几何裁剪 (update_submerged_area)：计算石片浸入水中的多边形形状及其面积 (Sim)，这是水动力计算的基础。

包含关键的坐标变换逻辑：处理石片的 自转 (Spin) 和 俯仰 (Pitch) 变换。

src/physics/derivative.rs:

实现 OdeSystem 特征，定义系统的微分方程。

核心受力分析 (compute_hydro_force)：

升力 (Lift)：垂直于速度方向，提供反弹力。

阻力 (Drag)：反向于速度，消耗能量。

垂直混合阻尼：结合线性阻尼和平方阻尼，抑制垂直方向的高频震荡。

表面张力吸附 (Suction)：模拟水面粘滞力，防止低速下的微小弹跳。

附加质量 (Added Mass)：模拟石片带动周围流体加速的惯性效应，显著提高模拟稳定性。

力矩计算 (compute_angular_acceleration)：计算水动力产生的力矩以及俯仰阻尼，防止石片剧烈翻转。

4. 数值求解器

src/solver2.rs:

实现通用的 四阶龙格-库塔法 (RK4) 求解器。

负责根据微分方程推演下一时刻的物理状态。

5. 渲染与可视化

src/print.rs:

基于 Macroquad 绘制模拟画面。

可视化石片轨迹、当前姿态、浸没状态。

提供即时的 UI 数据面板（速度、位置、角度等）。

支持视口的缩放和平移控制。

关键物理特性

本模拟特别针对“打水漂”这一复杂流体交互进行了优化：

非对称几何支持：支持任意形状的石片，通过精确的多边形裁剪计算浮力中心。

稳定的流体交互：引入 附加质量 (Added Mass) 和 混合阻尼 (Hybrid Damping)，有效解决了轻质物体在从高密度流体（水）中受力时的数值不稳定性（如飞出、剧烈震荡）。

能量守恒修正：修正了升阻力方向计算，并引入波辐射阻尼和水平水阻，确保系统的能量耗散符合物理规律。
