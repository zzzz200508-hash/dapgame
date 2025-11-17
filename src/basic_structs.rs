
//参考的基础结构定义。可以更方便的调用三维或者二维向量。四元数是否使用正在考虑
#[derive(Debug, Clone, Copy)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self{
            x: x,
            y: y,
            z: z,
        }
    }
}

impl Vector3D {
    fn length_squared(self) -> f64 { self.x * self.x + self.y * self.y + self.z * self.z}
    fn length(self) -> f64 { self.length_squared().sqrt() }
}
impl std::ops::Add for Vector3D {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output { Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z } }
}
impl std::ops::Sub for Vector3D {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output { Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z } }
}
impl std::ops::Mul<f64> for Vector3D {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output { Self { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs } }
}
impl std::ops::Div<f64> for Vector3D {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output { Self { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs } }
}

impl std::ops::Mul<Vector3D> for Vector3D {
    type Output = Self;
    fn mul(self, rhs: Vector3D) -> Self::Output { Self { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z } }
}
impl std::ops::Div<Vector3D> for Vector3D {
    type Output = Self;
    fn div(self, rhs: Vector3D) -> Self::Output { Self { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z } }
}

impl Vector3D {
    fn dot(self, rhs: Vector3D) -> f64 {
    self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    fn times(self, rhs: Vector3D) -> Self {
        Self{
            x: self.y * rhs.z - self.z * rhs.y,
            y: - self.x * rhs.z + self.z * rhs.x,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quaternion {
    pub fn identity() -> Self {
        Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 }
    }

    // 从角速度向量创建一个四元数 (0, vx, vy, vz)
    pub fn from_vector(v: Vector3D) -> Self {
        Self { w: 0.0, x: v.x, y: v.y, z: v.z }
    }

    // 归一化
    pub fn normalize(self) -> Self {
        let mag = (self.w*self.w + self.x*self.x + self.y*self.y + self.z*self.z).sqrt();
        if mag.abs() < 1e-10 {
            return Self::identity();
        }
        Self {
            w: self.w / mag,
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }
}

// 四元数乘法
impl std::ops::Mul for Quaternion {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }
}

impl std::ops::Mul<f64> for Quaternion {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self { w: self.w * rhs, x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}
impl std::ops::Add for Quaternion {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self { w: self.w + rhs.w, x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

// (用于转动惯量张量)
#[derive(Debug, Clone, Copy)]
pub struct Tensor3d {
    pub x: Vector3D,
    pub y: Vector3D,
    pub z: Vector3D
}
impl Tensor3d{
    fn new(x: Vector3D, y: Vector3D, z: Vector3D) -> Self{
        Self { x: x, y: y, z: z }
    }
}

// 石片（二维）存储点采用另一个结构处理
#[derive(Debug, Clone, Copy)]
pub struct Vector2D{
    pub x: f64,
    pub y: f64,
}

impl Vector2D{
    pub(crate) fn new(x: f64, y: f64) -> Self {
        Self{
            x: x,
            y: y
        }
    }
}

impl std::ops::Sub for Vector2D {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self { x: self.x - other.x, y: self.y - other.y }
    }
}
impl std::ops::Add for Vector2D {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}
impl std::ops::Mul<f64> for Vector2D {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}
impl Vector2D {
    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
    fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }
    fn length(self) -> f64 {
        self.length_squared().sqrt()
    }
    fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 { self * (1.0 / len) } else { self }
    }
}