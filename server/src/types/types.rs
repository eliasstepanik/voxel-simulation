use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Clone, Debug)]
pub struct DbVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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