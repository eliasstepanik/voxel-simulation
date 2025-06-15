use crate::plugins::environment::systems::voxels::structure::*;
use bevy::prelude::*;

/// Visualize each node of the octree as a scaled cuboid, **center-based**.
/// The octree's world-space center is `octree_tf.translation + octree.center`.
pub fn visualize_octree_system(
    mut gizmos: Gizmos,
    octree_query: Query<(&SparseVoxelOctree, &Transform)>,
) {
    for (octree, octree_tf) in octree_query.iter() {
        let root_center = octree_tf.translation + octree.center;

        // Draw the root bounding box
        gizmos.cuboid(
            Transform::from_translation(root_center).with_scale(Vec3::splat(octree.size)),
            Color::srgba(1.0, 1.0, 0.0, 0.15),
        );

        visualize_iterative(
            &mut gizmos,
            &octree.root,
            root_center,
            octree.size,
            octree.max_depth,
        );
    }
}

/// Recursively draws cuboids for each node.
/// We follow the same indexing as insert_recursive, i.e. bit patterns:
/// i=0 => child in (-x,-y,-z) quadrant,
/// i=1 => (+x,-y,-z), i=2 => (-x,+y,-z), etc.
fn visualize_iterative(
    gizmos: &mut Gizmos,
    root: &OctreeNode,
    root_center: Vec3,
    root_size: f32,
    max_depth: u32,
) {
    let mut stack = vec![(root, root_center, root_size, 0u32)];

    while let Some((node, center, size, depth)) = stack.pop() {
        if depth >= max_depth {
            continue;
        }

        if let Some(children) = &node.children {
            let child_size = size * 0.5;
            let half = child_size * 0.5;

            for (i, child) in children.iter().enumerate() {
                let offset_x = if (i & 1) != 0 { half } else { -half };
                let offset_y = if (i & 2) != 0 { half } else { -half };
                let offset_z = if (i & 4) != 0 { half } else { -half };

                let child_center = center + Vec3::new(offset_x, offset_y, offset_z);

                gizmos.cuboid(
                    Transform::from_translation(child_center).with_scale(Vec3::splat(child_size)),
                    Color::srgba(0.5, 1.0, 0.5, 0.15),
                );

                stack.push((child, child_center, child_size, depth + 1));
            }
        } else if node.is_leaf && node.voxel.is_some() {
            let leaf_size = size * 0.25;
            gizmos.cuboid(
                Transform::from_translation(center).with_scale(Vec3::splat(leaf_size)),
                Color::WHITE,
            );
        }
    }
}

#[allow(dead_code)]
pub fn draw_grid(
    mut gizmos: Gizmos,
    camera_query: Query<&Transform, With<Camera>>,
    octree_query: Query<(&SparseVoxelOctree, &Transform)>,
) {
    let Ok(camera_tf) = camera_query.get_single() else {
        return;
    };
    let camera_pos = camera_tf.translation;

    for (octree, octree_tf) in octree_query.iter() {
        let half_size = octree.size * 0.5;
        let root_center = octree_tf.translation + octree.center;

        // Voxel spacing at max depth
        let spacing = octree.get_spacing_at_depth(octree.max_depth);
        let grid_count = (octree.size / spacing) as i32;

        // We'll define the bounding region as [center-half_size .. center+half_size].
        // So the min corner is (root_center - half_size).
        let min_corner = root_center - Vec3::splat(half_size);

        // Draw lines in X & Z directions (like a ground plane).
        for i in 0..=grid_count {
            let offset = i as f32 * spacing;

            // 1) line along Z
            let x = min_corner.x + offset;
            let z1 = min_corner.z;
            let z2 = min_corner.z + (grid_count as f32 * spacing);

            let p1 = Vec3::new(x, min_corner.y, z1);
            let p2 = Vec3::new(x, min_corner.y, z2);

            // offset by -camera_pos for stable Gizmos in large coords
            let p1_f32 = p1 - camera_pos;
            let p2_f32 = p2 - camera_pos;
            gizmos.line(p1_f32, p2_f32, Color::WHITE);

            // 2) line along X
            let z = min_corner.z + offset;
            let x1 = min_corner.x;
            let x2 = min_corner.x + (grid_count as f32 * spacing);

            let p3 = Vec3::new(x1, min_corner.y, z) - camera_pos;
            let p4 = Vec3::new(x2, min_corner.y, z) - camera_pos;
            gizmos.line(p3, p4, Color::WHITE);
        }
    }
}
