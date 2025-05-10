use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Clone, Debug)]
pub struct DbVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl DbVector3 {
    pub(crate) fn new(x: f32, y: f32, z: f32) -> DbVector3 {
        DbVector3 { x, y, z }
    }
    pub(crate) fn zero() -> DbVector3 {
        DbVector3::new(0.0, 0.0, 0.0)
    }
    pub(crate) fn add(&mut self, other: &DbVector3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
    pub(crate) fn mul_scalar(&self, s: f32) -> DbVector3 {
        DbVector3::new(self.x * s, self.y * s, self.z * s)
    }
}
#[derive(SpacetimeType, Clone, Debug)]
pub struct DBVector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}


#[derive(SpacetimeType, Clone, Debug)]
pub struct DbTransform {
    pub position: DbVector3,
    pub rotation: DBVector4,
    pub scale: DbVector3,
}