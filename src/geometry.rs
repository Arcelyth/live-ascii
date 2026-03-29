#![allow(dead_code)]

#[derive(Debug, Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32, 
    pub z: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub v1: Vec3,
    pub v2: Vec3,
    pub v3: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    pub minx: f32,
    pub miny: f32,
    pub maxx: f32,
    pub maxy: f32,
}

impl Triangle {
    pub fn new(v1: Vec3, v2: Vec3, v3: Vec3) -> Self {
        Self { v1, v2, v3 }
    }

    pub fn get_box(&self) -> BoundingBox {
        let minx = self.v1.x.min(self.v2.x).min(self.v3.x);
        let miny = self.v1.y.min(self.v2.y).min(self.v3.y);
        let maxx = self.v1.x.max(self.v2.x).max(self.v3.x);
        let maxy = self.v1.y.max(self.v2.y).max(self.v3.y);
        BoundingBox {
            minx, miny, maxx, maxy
        }                  
    }

    pub fn signed_area(&self) -> f32 {
        return 0.5 * ((self.v2.x - self.v1.x) * (self.v3.y - self.v1.y) - (self.v2.y - self.v1.y) * (self.v3.x - self.v1.x))
    }

}


