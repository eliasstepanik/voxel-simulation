use bevy::prelude::Resource;
use serde::Deserialize;

#[derive(Debug, Deserialize, Resource)]
pub struct Config {
    pub server: ServerConfig,
}


#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub database: String,
    
}