

use std::time::Duration;
use spacetimedb::{ReducerContext, ScheduleAt, Table};

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    log::info!("Initializing...");

}