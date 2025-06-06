use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::*;
use big_space::floating_origins::FloatingOrigin;
use big_space::prelude::GridCell;
use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::camera_system::CameraController;

pub struct PlanetMaker {}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    root:          Res<RootGrid>
) {
    // Four corner vertices – y is up in Bevy’s 3-D coordinate system
    let positions = vec![
        [-0.5, 0.0, -0.5],
        [ 0.5, 0.0, -0.5],
        [ 0.5, 0.0,  0.5],
        [-0.5, 0.0,  0.5],
    ];

    // Single normal for all vertices (pointing up)
    let normals = vec![[0.0, 1.0, 0.0]; 4];

    // UVs for a full-size texture
    let uvs = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];

    // Two triangles: (0,1,2) and (0,2,3)
    let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

    let plane = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(indices);

    let mesh_handle = meshes.add(plane);
    let material_handle = materials.add(StandardMaterial::from_color(Color::srgb(0.3, 0.6, 1.0)));


    let sphere = meshes.add(
        SphereMeshBuilder::new(1.0, SphereKind::Ico { subdivisions: 5 })
            .build()
    );



    commands.entity(root.0).with_children(|parent| {
        
        parent.spawn((
            Name::new("Planet"),
            Mesh3d(sphere),
            MeshMaterial3d(material_handle),
            GridCell::<i64>::ZERO,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ));
    });
    
}