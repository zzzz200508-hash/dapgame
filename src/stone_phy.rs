use crate::basic_structs::Vector2D;
use crate::stone_editor::StoneBlueprint;

// --- 常量 ---
/// 石片碰撞网格的采样点数 (近似)
const COLLISION_MESH_POINTS: usize = 40000;
/// 石头的密度 (假设为板岩, kg/m^3)
const DENSITY_SLATE: f64 = 2700.0;

/// 石片物理属性
#[derive(Clone)]
pub struct StoneProperties {
    pub n: usize,// 碰撞点云总点数
    /// 石片总质量 (kg)
    pub mass: f64,

    /// 石片转动惯量 (I)
    pub inertia_tensor_x: f64,// 方向垂直纸面
    pub inertia_tensor_y: f64,// 自转转动惯量

    /// 质心坐标系下的轮廓点 (m)
    pub outline_com: Vec<Vector2D>,

    /// 质心坐标系下的碰撞点云 (m)
    pub collision_mesh_com: Vec<Vector2D>,

    pub d_max: f64,// 距离质心最远点(用于快速判断是否进水)
}

impl StoneProperties {
    /// 1. 计算面积和质心
    /// 2. 计算质量
    /// 3. 将轮廓平移到质心系
    /// 4. 生成质心系下的碰撞点云
    /// 5. 计算转动惯量
    pub fn new(blueprint: &StoneBlueprint) -> Self {
        // 1. 计算面积
        let area = calculate_polygon_area(&blueprint.points);
        if area.abs() < 1e-9 {
            println!("Warning: Stone area is near zero.");
            return Self::default(); // 返回一个空/默认的石头
        }

        // 2. 计算质心
        let centroid = calculate_centroid(&blueprint.points);

        // 3. 计算质量
        let mass = area.abs() * blueprint.thickness * DENSITY_SLATE;

        // 4. 将轮廓平移到质心系
        let outline_com: Vec<Vector2D> = blueprint.points.iter()
            .map(|p| *p - centroid)
            .collect();

        // 5. 生成质心系下的碰撞点云
        //    (我们在已经平移的轮廓内采样)
        let collision_mesh_com = generate_collision_mesh(&outline_com, COLLISION_MESH_POINTS, area);

        // 6. 计算转动惯量
        let inertia_tensor_x = calculate_inertia_z(&collision_mesh_com, mass);
        let inertia_tensor_y = calculate_inertia_y(&collision_mesh_com, mass);

        let n = collision_mesh_com.len();

        let mut d_max = 0.0;
        for point in & collision_mesh_com{
            if point.x * point.x + point.y * point.y >= d_max {d_max = point.x * point.x + point.y * point.y}
            else{d_max = d_max}
        }

        // 7. 返回最终的物理对象
        Self {
            n,// 碰撞点云总点数
            mass,// 质量
            inertia_tensor_x,// 垂直纸面转动惯量
            inertia_tensor_y,// 石片自旋转动惯量
            outline_com,// 质心系下石片边界
            collision_mesh_com,// 质心系下所有碰撞点
            d_max,// 距离质心最远点(用于快速判断是否进水)
        }
    }
}

// (为面积为 0 时提供安全的回退)
impl Default for StoneProperties {
    fn default() -> Self {
        Self {
            n: 0,
            mass: 0.0,
            inertia_tensor_x: 0.0,
            inertia_tensor_y: 0.0,
            outline_com: vec![],
            collision_mesh_com: vec![],
            d_max: 0.0,
        }
    }
}


// --- 几何与物理计算辅助函数 ---

/// 计算多边形面积
/// 使用 Shoelace (鞋带) 公式
fn calculate_polygon_area(polygon: &[Vector2D]) -> f64 {
    if polygon.len() < 3 { return 0.0; }

    let mut area = 0.0;
    let n = polygon.len();
    for i in 0..n {
        let p1 = polygon[i];
        let p2 = polygon[(i + 1) % n]; // 环绕到第一个点
        area += p1.x * p2.y - p2.x * p1.y;
    }

    // 面积可为负 (取决于顶点顺序), 取 0.5 * 绝对值
    (area / 2.0).abs()
}

/// 计算多边形质心 (解析法)
fn calculate_centroid(polygon: &[Vector2D]) -> Vector2D {
    let mut centroid_x = 0.0;
    let mut centroid_y = 0.0;
    let n = polygon.len();

    // (这个公式要求面积是"有符号"的)
    let mut signed_area_x2 = 0.0; // 2 * A

    for i in 0..n {
        let p1 = polygon[i];
        let p2 = polygon[(i + 1) % n];

        let cross_product = p1.x * p2.y - p2.x * p1.y;
        signed_area_x2 += cross_product;

        centroid_x += (p1.x + p2.x) * cross_product;
        centroid_y += (p1.y + p2.y) * cross_product;
    }

    if signed_area_x2.abs() < 1e-9 {
        // 如果面积为0 (例如一条直线), 就返回几何中心
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        for p in polygon {
            sum_x += p.x;
            sum_y += p.y;
        }
        return Vector2D::new(sum_x / n as f64, sum_y / n as f64);
    }

    // (除以 6 * A)
    let factor = 1.0 / (3.0 * signed_area_x2);
    Vector2D::new(centroid_x * factor, centroid_y * factor)
}

/// 寻找 AABB
/// 返回 (min_corner, max_corner)
fn find_aabb(polygon: &[Vector2D]) -> (Vector2D, Vector2D) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for p in polygon {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    (Vector2D::new(min_x, min_y), Vector2D::new(max_x, max_y))
}

/// 射线法 (Ray Casting) 判断点是否在多边形内
fn is_point_in_polygon(point: Vector2D, polygon: &[Vector2D]) -> bool {
    let mut is_inside = false;
    let n = polygon.len();
    let mut j = n - 1; // 最后一个顶点

    for i in 0..n {
        let pi = polygon[i];
        let pj = polygon[j];

        // 核心算法
        let intersects = ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x);

        if intersects {
            is_inside = !is_inside;
        }
        j = i; // j 追随 i
    }
    is_inside
}

/// 生成碰撞点云
/// 使用网格采样法 (Grid Sampling)
fn generate_collision_mesh(polygon: &[Vector2D], num_points: usize, polygon_area: f64) -> Vec<Vector2D> {
    if polygon.is_empty() || polygon_area.abs() < 1e-9 { return Vec::new(); }

    let (min, max) = find_aabb(polygon);
    let aabb_width = max.x - min.x;
    let aabb_height = max.y - min.y;

    if aabb_width.abs() < 1e-9 || aabb_height.abs() < 1e-9 { return Vec::new(); }

    // --- 网格计算 ---
    // 1. 计算每个点代表的面积
    let area_per_point = polygon_area / (num_points as f64);
    // 2. 计算网格间距 (delta)
    let delta = area_per_point.sqrt();

    // 3. 计算需要检查的行列数
    let num_cols = (aabb_width / delta).ceil() as usize + 1;
    let num_rows = (aabb_height / delta).ceil() as usize + 1;

    let mut mesh = Vec::new();

    // 4. 遍历网格
    for i in 0..num_rows {
        let y = min.y + i as f64 * delta;
        // 优化：如果整行都在AABB之外，则跳过
        if y > max.y { continue; }

        for j in 0..num_cols {
            let x = min.x + j as f64 * delta;
            // 优化：如果点在AABB之外，则跳过
            if x > max.x { continue; }

            let point = Vector2D::new(x, y);

            // 5. 检查网格点是否在多边形内
            if is_point_in_polygon(point, polygon) {
                mesh.push(point);
            }
        }
    }

    mesh
}


/// 计算转动惯量 (I_z)
///
/// 2D模拟中的转动惯量是绕 Z 轴 (垂直于平面) 的标量。
/// I_z = Σ (m_i * r_i^2) = Σ m_i * (x_i^2 + y_i^2)
///
/// `mesh_points`: 必须是质心坐标系下的点
/// `total_mass`: 石片总质量
fn calculate_inertia_z(mesh_points: &[Vector2D], total_mass: f64) -> f64 {
    let n = mesh_points.len();
    if n == 0 { return 0.0; }

    // 假设质量均匀分布, 每个采样点的质量
    let mass_per_point = total_mass / (n as f64);

    let mut inertia_sum = 0.0;

    for point in mesh_points {
        let r_squared = point.x * point.x;
        inertia_sum += r_squared; // 我们先把 Σ(r_i^2) 加起来
    }

    // I_z = Σ(m_i * r_i^2) = m_i * Σ(r_i^2)
    mass_per_point * inertia_sum
}

fn calculate_inertia_y(mesh_points: &[Vector2D], total_mass: f64) -> f64 {
    let n = mesh_points.len();
    if n == 0 { return 0.0; }

    // 假设质量均匀分布, 每个采样点的质量
    let mass_per_point = total_mass / (n as f64);

    let mut inertia_sum = 0.0;

    for point in mesh_points {
        let r_squared = point.x * point.x + point.y * point.y;
        inertia_sum += r_squared; // 我们先把 Σ(r_i^2) 加起来
    }

    // I_z = Σ(m_i * r_i^2) = m_i * Σ(r_i^2)
    mass_per_point * inertia_sum
}