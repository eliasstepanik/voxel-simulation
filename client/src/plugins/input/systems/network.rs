
use bevy::input::ButtonInput;
use bevy::math::{EulerRot, Quat};
use bevy::prelude::{KeyCode, Res, ResMut,};
use random_word::Lang;
use crate::module_bindings::{set_name, spawn_entity, DbTransform, DbVector3, DbVector4};
use crate::plugins::network::systems::database::DbConnectionResource;

pub fn network_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    ctx: ResMut<DbConnectionResource>,
) {
    let word = random_word::get(Lang::En);

    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        ctx.0.reducers.set_name(word.to_string()).unwrap();
    }
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        let rand_position = crate::helper::vector_helper::random_vec3(-10.0, 10.0);
        let rand_rotation = crate::helper::vector_helper::random_vec3(0.0, 10.0);
        let rand_rotation = Quat::from_euler(EulerRot::XYZ,rand_rotation.x,rand_rotation.y,rand_rotation.z).normalize();
        let rand_scale = crate::helper::vector_helper::random_vec3(0.1, 1.0);
        ctx.0.reducers.spawn_entity(DbTransform{
            position: DbVector3{
                x: rand_position.x,
                y: rand_position.y,
                z: rand_position.z,
            },
            rotation: DbVector4 {
                x: rand_rotation.x,
                y: rand_rotation.y,
                z: rand_rotation.z,
                w: rand_rotation.w,
            },
            
            
            
            scale: DbVector3 {
                x: rand_scale.x,
                y: rand_scale.x,
                z: rand_scale.x,
            },
        }).unwrap();
    }
    
}

