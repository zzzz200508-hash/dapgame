use crate::basic_structs::Vector2D;
//采用两种方法获得石头片的样子：通过控制多个点绘制贝塞尔曲线或通过直接绘图。


// 总计算点数
pub const CALCULATE_POINTS: usize = 1000;

pub struct BezierInfo{
    name: String,
    control_points: Vec<Vector2D>,
    order: usize,
    pub polyline_points: Vec<Vector2D>,
}

impl BezierInfo {
    pub(crate) fn new(name: String, points: Vec<Vector2D>) -> Self {
        let order = if points.len() > 0 { points.len() - 1 } else { 0 };

        let resolution = CALCULATE_POINTS; //计算的点数
        let mut polyline_points = Vec::with_capacity(resolution + 1);
        if !points.is_empty() {
            for i in 0..=resolution {
                let t = i as f64 / resolution as f64;
                if let Some(point) = de_iterative(&points, t) {
                    polyline_points.push(point);
                }
            }
        }
        Self {
            name: name,
            order: order,
            control_points: points,
            polyline_points: polyline_points, // 存储结果
        }
    }

    pub(crate) fn get_polyline_points(&self) -> &Vec<Vector2D> {
        &self.polyline_points
    }
}

pub fn lerp(a: Vector2D, b: Vector2D, t: f64) -> Vector2D {
    Vector2D {
        x: (1.0 - t) * a.x + t * b.x,
        y: (1.0 - t) * a.y + t * b.y,
    }
}

//使用德卡斯特里奥算法计算 n 阶贝塞尔曲线在 t 处的值。
//`points`: 包含 n+1 个控制点的切片。
//`t`: 参数，通常在 [0.0, 1.0] 范围内。

pub fn de_iterative(points: &[Vector2D], t: f64) -> Option<Vector2D> {

    if points.is_empty() {
        return None;
    }

    let mut buffer = points.to_vec();

    //获取当前迭代的点数
    let mut n = buffer.len();

    while n > 1 {
        //每次内层循环计算一个新的插值点
        for i in 0..(n - 1) {
            //在原地更新 buffer[i] 为它与下一个点的插值
            buffer[i] = lerp(buffer[i], buffer[i + 1], t);
        }
        //点的数量减少 1 (上升一层)
        n -= 1;
    }

    Some(buffer[0])
}