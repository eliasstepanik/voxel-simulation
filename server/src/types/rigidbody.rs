use std::time::Duration;
use spacetimedb::{ReducerContext, Table, TimeDuration, Timestamp};
use crate::{physics_timer, PhysicsTimer};
use crate::types::entity::{entity, Entity, EntityType};
use crate::types::types::{DbTransform, DbVector3};

#[spacetimedb::table(name = rigidbody, public)]
#[derive(Debug, Clone, )]
pub struct Rigidbody {
    #[auto_inc]
    #[primary_key]
    pub rigidbody_id: u32,

    #[index(btree)]
    pub entity_id: u32,


    pub velocity: DbVector3,
    pub force: DbVector3,
    pub mass: f32,

    pub is_fixed: bool,
}

#[spacetimedb::reducer]
pub fn spawn_rigidbody_entity(
    ctx: &ReducerContext,
    transform: DbTransform,
    entity_type: EntityType,
    velocity: DbVector3,
    mass: f32,
    is_fixed: bool,
) -> Result<(), String> {
    // 1. insert a new Entity row
    let inserted_entity: Entity = ctx
        .db
        .entity()
        .insert(Entity {
            entity_id: 0,
            transform,
            entity_type,
        });

    // 2. insert its corresponding Rigidbody row
    ctx.db
        .rigidbody()
        .insert(Rigidbody {
            rigidbody_id: 0,
            entity_id: inserted_entity.entity_id,
            velocity,
            force: DbVector3::zero(),
            mass,
            is_fixed,
        });

    Ok(())
}



#[spacetimedb::reducer]
pub fn physics_step(ctx: &ReducerContext, mut timer: PhysicsTimer) -> Result<(), String> {
    let now = ctx.timestamp;
    let delta = now
        .time_duration_since(timer.last_update_ts)
        .unwrap_or(TimeDuration::from(Duration::from_millis(50)));
    let dt = delta.to_duration().unwrap().as_secs_f32();

    // update timer state
    timer.last_update_ts = now;
    ctx.db.physics_timer().scheduled_id().update(timer);

    // constant gravity
    let gravity = DbVector3::new(0.0, -9.81, 0.0);

    // process all rigidbodies
    let bodies: Vec<Rigidbody> = ctx.db.rigidbody().iter().collect();
    for mut rb in bodies {
        if rb.is_fixed {
            continue;
        }

        // apply gravity to force
        rb.force.add(&gravity.mul_scalar(rb.mass));

        // integrate velocity
        let inv_mass = 1.0 / rb.mass;
        let accel = rb.force.mul_scalar(inv_mass);
        rb.velocity.add(&accel.mul_scalar(dt));

        // update corresponding entity position
        if let Some(mut ent) = ctx.db.entity().iter().find(|e| e.entity_id == rb.entity_id){
            ent.transform.position.add(&rb.velocity.mul_scalar(dt));
            ctx.db.entity().entity_id().update(ent);
        }

        // reset force and write back
        rb.force = DbVector3::zero();
        ctx.db.rigidbody().rigidbody_id().update(rb);
    }

    Ok(())
}