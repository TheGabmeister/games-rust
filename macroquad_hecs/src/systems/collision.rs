use std::cell::RefCell;

use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::components::{
    ActivePowerup, BoxCollider, CircleCollider, CollisionLayer, Enemy, Pickup, Transform,
};
use crate::constants::*;
use crate::events::{EventBus, GameEvent};



// ---------------------------------------------------------------------------
// Main collision system
// ---------------------------------------------------------------------------

pub fn system_collision(world: &mut World, events: &mut EventBus) {

}
