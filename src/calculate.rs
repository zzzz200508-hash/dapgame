use crate::basic_structs::Vector2D;
use crate::solver2::{OdeSystem, VectorSpace};
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

pub struct CustomSettings{
    pub gravity:f64
    // 其他可能需要的参数都可以写在这里，作为对StoneInfo的补充
}

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

impl CustomSettings{
    pub(crate) fn new(g:f64 ) -> Self{
        CustomSettings{
        gravity: g,

        }
    }
}

impl OdeSystem<StoneInfo> for CustomSettings {
    fn derivatives(&self, _t: f64, y: &StoneInfo) -> StoneInfo {
        StoneInfo {
            position: y.position,
            velocity: y.velocity,
            angle: y.angle,
            angle_velocity: y.angle_velocity,
            //=====
            //这里是计算导数区域,数学部分到此结束,这里也是后面物理模型需要更改的地方,但很显然还没写完.总而言之这样就可以把这个小B玩意撇到求解器里了.
            //=====
        }
    }
}


//这一段颜子涵还没写。我拉了一些ai示例如下。是上次的海伯利安模拟
#[derive(Clone, Debug)]
pub struct HyperionState {
    pub theta: f64,
    pub omega: f64,
    // 你甚至可以在这里混用不同单位，只要数学上说得通
}

impl VectorSpace for HyperionState {
    fn add(&self, other: &Self) -> Self {
        Self {
            theta: self.theta + other.theta,
            omega: self.omega + other.omega,
        }
    }

    fn scale(&self, scalar: f64) -> Self {
        Self {
            theta: self.theta * scalar,
            omega: self.omega * scalar,
        }
    }
}

// 示例 1：使用 Vec<f64> (动态维度)
struct SimpleHarmonicOscillator;

impl OdeSystem<Vec<f64>> for SimpleHarmonicOscillator {
    fn derivatives(&self, _t: f64, y: &Vec<f64>) -> Vec<f64> {
        // y[0] = x, y[1] = v
        // dx/dt = v, dv/dt = -x
        vec![y[1], -y[0]]
    }
}

// 示例 2：使用自定义结构体 (静态维度，强类型，性能更高)
pub struct CustomOscillator;

impl OdeSystem<HyperionState> for CustomOscillator {
    fn derivatives(&self, _t: f64, y: &HyperionState) -> HyperionState {
        HyperionState {
            theta: y.omega,
            omega: -y.theta, // 简谐运动示例
        }
    }
}
