use std::any::{Any, TypeId};
use std::collections::HashMap;

use hecs::Entity;

use crate::components::{EnemyKind, PickupKind, PowerupEffect};
use crate::managers::{GameDirector, MusicManager, SfxManager};
use crate::resources::DespawnQueue;

// ---------------------------------------------------------------------------
// Audio IDs/commands — placed here so gameplay and audio are in one import.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SfxId {
    PlayerLaser,
    PlayerDied,
    PlayerPowerup,
    EnemyLaser,
    EnemyDestroyed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MusicId {
    Spaceshooter,
}

// ---------------------------------------------------------------------------
// Game event trait + concrete event structs
// ---------------------------------------------------------------------------

pub trait GameEvent: 'static + Any {}

#[derive(Debug)]
pub struct GameStarted;
impl GameEvent for GameStarted {}

#[derive(Debug)]
pub struct PlayerDied;
impl GameEvent for PlayerDied {}

#[derive(Debug)]
#[allow(dead_code)] // Fields form the event's data contract for current + future handlers.
pub struct EnemyDestroyed {
    pub entity: Entity,
    pub kind: EnemyKind,
}
impl GameEvent for EnemyDestroyed {}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PickupCollected {
    pub entity: Entity,
    pub kind: PickupKind,
}
impl GameEvent for PickupCollected {}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PowerupCollected {
    pub entity: Entity,
    pub player: Entity,
    pub effect: PowerupEffect,
    pub duration: f32,
}
impl GameEvent for PowerupCollected {}

#[derive(Debug)]
pub struct StageCleared;
impl GameEvent for StageCleared {}

#[derive(Debug)]
pub struct PlaySfx {
    pub id: SfxId,
}
impl GameEvent for PlaySfx {}

#[derive(Debug)]
pub struct PlayMusic {
    pub id: MusicId,
}
impl GameEvent for PlayMusic {}

// ---------------------------------------------------------------------------
// Handler context — bundles all mutable resources a handler might need.
// ---------------------------------------------------------------------------

pub struct EventContext<'a> {
    pub world: &'a mut hecs::World,
    pub director: &'a mut GameDirector,
    pub despawns: &'a mut DespawnQueue,
    pub sfx: &'a mut SfxManager,
    pub music: &'a mut MusicManager,
    pub(crate) deferred: Vec<ErasedEvent>,
}

impl EventContext<'_> {
    /// Emit a follow-up event from within a handler.
    /// It will be dispatched after the current event's handlers finish.
    pub fn emit<E: GameEvent>(&mut self, event: E) {
        self.deferred.push(ErasedEvent::new(event));
    }
}

// ---------------------------------------------------------------------------
// Type-erased event storage
// ---------------------------------------------------------------------------

pub struct ErasedEvent {
    pub type_id: TypeId,
    pub event: Box<dyn Any>,
}

impl ErasedEvent {
    pub fn new<E: GameEvent>(event: E) -> Self {
        Self {
            type_id: TypeId::of::<E>(),
            event: Box::new(event),
        }
    }
}

// ---------------------------------------------------------------------------
// Event queue — stores pending events (type-erased)
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct EventQueue {
    queue: Vec<ErasedEvent>,
}

impl EventQueue {
    pub fn emit<E: GameEvent>(&mut self, event: E) {
        self.queue.push(ErasedEvent::new(event));
    }

    pub fn drain_raw(&mut self) -> Vec<ErasedEvent> {
        std::mem::take(&mut self.queue)
    }
}

// ---------------------------------------------------------------------------
// Event registry — maps event types to handler functions
// ---------------------------------------------------------------------------

type ErasedHandler = Box<dyn Fn(&dyn Any, &mut EventContext)>;

#[derive(Default)]
pub struct EventRegistry {
    handlers: HashMap<TypeId, Vec<ErasedHandler>>,
}

impl EventRegistry {
    pub fn on<E: GameEvent>(&mut self, handler: impl Fn(&E, &mut EventContext) + 'static) {
        let type_id = TypeId::of::<E>();
        let erased: ErasedHandler = Box::new(move |any, ctx| {
            let event = any.downcast_ref::<E>().unwrap();
            handler(event, ctx);
        });
        self.handlers.entry(type_id).or_default().push(erased);
    }

    pub fn dispatch(&self, event: &ErasedEvent, ctx: &mut EventContext) {
        if let Some(handlers) = self.handlers.get(&event.type_id) {
            for handler in handlers {
                handler(&*event.event, ctx);
            }
        }
    }
}
