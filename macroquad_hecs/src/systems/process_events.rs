use std::collections::VecDeque;

use hecs::Entity;
use hecs::World;

use crate::components::Enemy;
use crate::events::{ErasedEvent, EventContext, EventQueue, EventRegistry, StageCleared};
use crate::managers::{GameDirector, MusicManager, SfxManager};
use crate::resources::{DespawnQueue, GameState};

fn stage_cleared(world: &World, despawns: &DespawnQueue) -> bool {
    world
        .query::<(Entity, &Enemy)>()
        .iter()
        .all(|(entity, _)| despawns.contains(entity))
}

// ---------------------------------------------------------------------------
// Process events — type-erased dispatch to registered handlers
// ---------------------------------------------------------------------------

pub fn system_process_events(
    world: &mut World,
    events: &mut EventQueue,
    registry: &EventRegistry,
    director: &mut GameDirector,
    despawns: &mut DespawnQueue,
    sfx: &mut SfxManager,
    music: &mut MusicManager,
) {
    let mut pending: VecDeque<ErasedEvent> = events.drain_raw().into();
    let mut stage_clear_queued = false;

    while let Some(event) = pending.pop_front() {
        let deferred = {
            let mut ctx = EventContext {
                world,
                director,
                despawns,
                sfx,
                music,
                deferred: Vec::new(),
            };
            registry.dispatch(&event, &mut ctx);
            ctx.deferred
        };

        // Append any follow-up events emitted by handlers.
        for ev in deferred {
            pending.push_back(ev);
        }

        if !stage_clear_queued
            && director.state == GameState::Playing
            && stage_cleared(world, despawns)
        {
            pending.push_back(ErasedEvent::new(StageCleared));
            stage_clear_queued = true;
        }
    }
}
