use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::ecs::prelude::ChildOf;

use big_space::prelude::*;

/// Plugin enabling high precision coordinates using `big_space`.
///
/// This sets up [`BigSpacePlugin`] so entities can be placed far from the origin
/// without losing precision.
// ── plugin that creates the grid ──────────────────────────────────────────────
pub struct BigSpaceIntegrationPlugin;

#[derive(Resource)]
pub struct RootGrid(pub Entity);

impl Plugin for BigSpaceIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigSpacePlugin::<i64>::default());

        app.add_systems(PreStartup, (spawn_root, cache_root.after(spawn_root)));
        app.add_systems(PostStartup, (fix_invalid_children));

        app.add_systems(PostUpdate,(fix_invalid_children));
    }
}

// 1) build the Big-Space root
fn spawn_root(mut commands: Commands) {
    commands.spawn_big_space_default::<i64>(|_| {});
}

// 2) cache the root entity for later use
fn cache_root(
    mut commands: Commands,
    roots: Query<Entity, (With<BigSpace>, Without<ChildOf>)>,   // top-level grid
) {
    if let Ok(entity) = roots.get_single() {

        commands.entity(entity).insert(Visibility::Visible);
        commands.insert_resource(RootGrid(entity));
    }
}

fn fix_invalid_children(
    mut commands: Commands,
    bad: Query<Entity, (With<FloatingOrigin>, Without<GridCell<i64>>, With<ChildOf>)>,
) {
    for e in &bad {
        commands.entity(e).insert(GridCell::<i64>::ZERO);
    }
}


pub fn move_by(
    mut q: Query<&mut Transform>,
    delta: Vec3,          // metres inside the current cell
) {
    for mut t in &mut q {
        t.translation += delta;       // small numbers only
    }
}



pub fn teleport_to<P: GridPrecision>(
    e: Entity,
    target: DVec3,
    grids: Grids<'_, '_, P>,
    mut q: Query<(&ChildOf, &mut GridCell<P>, &mut Transform)>,
) {
    let (child_of, mut cell, mut tf) = q.get_mut(e).unwrap();
    let grid = grids.parent_grid(child_of.parent()).unwrap();

    let (new_cell, local) = grid.translation_to_grid(target);

    *cell = new_cell;
    tf.translation = local;
}
