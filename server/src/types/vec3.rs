use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Clone, Debug)]
pub struct DbVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[spacetimedb::table(name = entity, public)]
#[derive(Debug, Clone)]
pub struct Entity {
    // The `auto_inc` attribute indicates to SpacetimeDB that
    // this value should be determined by SpacetimeDB on insert.
    #[auto_inc]
    #[primary_key]
    pub entity_id: u32,
    pub position: DbVector3,
    pub mass: u32,
}