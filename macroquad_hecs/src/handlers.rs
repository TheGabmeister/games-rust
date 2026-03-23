use crate::events::{
    EnemyDestroyed, EventContext, GameStarted, PickupCollected, PlaySfx, PlayerDied,
    PowerupCollected, SfxId, StageCleared,
};

// ---------------------------------------------------------------------------
// Audio handler — the only handler that touches SfxManager
// ---------------------------------------------------------------------------

pub fn on_play_sfx(event: &PlaySfx, ctx: &mut EventContext) {
    ctx.sfx.play_sound(event.id);
}

// ---------------------------------------------------------------------------
// Gameplay handlers — emit PlaySfx instead of calling sfx directly
// ---------------------------------------------------------------------------

pub fn on_game_started(_event: &GameStarted, _ctx: &mut EventContext) {
    // Currently a no-op (music play is commented out upstream).
}

pub fn on_enemy_destroyed(event: &EnemyDestroyed, ctx: &mut EventContext) {
    ctx.director.on_enemy_destroyed(ctx.world, event.entity);
    ctx.emit(PlaySfx {
        id: SfxId::EnemyDestroyed,
    });
}

pub fn on_player_died(_event: &PlayerDied, ctx: &mut EventContext) {
    ctx.director.on_player_died(ctx.world, ctx.despawns);
    ctx.emit(PlaySfx {
        id: SfxId::PlayerDied,
    });
}

pub fn on_pickup_collected(event: &PickupCollected, ctx: &mut EventContext) {
    ctx.director.apply_pickup_reward(event.kind);
}

pub fn on_powerup_collected(event: &PowerupCollected, ctx: &mut EventContext) {
    ctx.director
        .apply_powerup(ctx.world, event.player, event.effect, event.duration);
    ctx.emit(PlaySfx {
        id: SfxId::PlayerPowerup,
    });
}

pub fn on_stage_cleared(_event: &StageCleared, ctx: &mut EventContext) {
    ctx.director.on_stage_cleared();
}
