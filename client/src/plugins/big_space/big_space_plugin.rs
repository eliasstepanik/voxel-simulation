use bevy::prelude::*;
use big_space::prelude::*;

/// Plugin enabling high precision coordinates using `big_space`.
///
/// This sets up [`BigSpacePlugin`] so entities can be placed far from the origin
/// without losing precision.
pub struct BigSpaceIntegrationPlugin;

impl Plugin for BigSpaceIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigSpacePlugin::<i64>::default());
    }
}
