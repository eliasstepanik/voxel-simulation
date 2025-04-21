use bevy::math::Vec3;
use bevy::prelude::{Quat, Transform};
use rand::Rng;
use crate::helper::math::RoundTo;
use crate::module_bindings::DbTransform;

pub(crate) fn random_vec3(min: f32, max: f32) -> Vec3 {
    let mut rng = rand::thread_rng();
    Vec3::new(
        rng.gen_range(min..max),
        rng.gen_range(min..max),
        rng.gen_range(min..max),
    )
}

impl From<DbTransform> for Transform {
    fn from(db: DbTransform) -> Self {
        Transform {
            translation: Vec3::new(db.position.x, db.position.y, db.position.z),
            rotation: //Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
            Quat::from_xyzw(
                db.rotation.x.round_to(3),
                db.rotation.y.round_to(3),
                db.rotation.z.round_to(3),
                db.rotation.w.round_to(3),
            ),
            scale: Vec3::new(db.scale.x.round_to(3), db.scale.y.round_to(3), db.scale.z.round_to(3)),
        }
    }
}