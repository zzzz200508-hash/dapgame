
use crate::stone_phy::StoneProperties; 
use crate::basic_structs::Vector2D; 
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Phase {
    Flying, 
    Bouncing, 
    Sinking
}

pub struct CustomSettings {
    pub gravity: f64,
    pub rho: f64,
    pub Cl: f64,
    pub Cf: f64,
    pub Sim: f64,
    pub M: f64,
    pub beta: f64,
    pub phase: Phase,
    pub water_level: f64,

    pub stone: StoneProperties, 
    pub current_submerged_polygon: Vec<Vector2D>,
}


impl CustomSettings{
    pub(crate) fn new(g:f64, stone: StoneProperties ) -> Self{
        CustomSettings{
        gravity: g, 
        rho: 1000.0,         // 水的密度 (kg/m^3)
        Cl: 0.2,             // 默认升力系数
        Cf: 0.05,            // 默认摩擦/阻力系数
        Sim: 0.01,           // 石头横截面积 (m^2)            
        M: stone.mass,              // 石头质量 (kg)
        beta: 0.02,          // 旋转阻尼
        phase: Phase::Flying, 
        water_level: 0.0,

        stone: stone, 
        current_submerged_polygon: Vec::new(), 

        }
    }
}

