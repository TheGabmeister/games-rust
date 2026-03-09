use crate::components::{EnemyKind, EnemyMode};
use crate::resources::StageType;

#[derive(Clone, Copy, Debug)]
pub struct DifficultyTuning {
    pub dive_interval: f32,
    pub max_divers: usize,
    pub enemy_fire_interval: f32,
    pub enemy_bullet_speed: f32,
    pub dive_speed_multiplier: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapturedShipOutcome {
    RescueDual,
    BetrayLater,
}

pub fn stage_type_for(stage: u32) -> StageType {
    if stage >= 3 && (stage - 3) % 4 == 0 {
        StageType::Challenge
    } else {
        StageType::Normal
    }
}

pub fn enemy_base_hp(kind: EnemyKind) -> i32 {
    match kind {
        EnemyKind::BossGalaga => 2,
        _ => 1,
    }
}

pub fn score_for_enemy(kind: EnemyKind, was_diving: bool, stage_type: StageType) -> u32 {
    if stage_type == StageType::Challenge {
        return 100;
    }

    match (kind, was_diving) {
        (EnemyKind::Bee, false) => 50,
        (EnemyKind::Bee, true) => 100,
        (EnemyKind::Butterfly, false) => 80,
        (EnemyKind::Butterfly, true) => 160,
        (EnemyKind::BossGalaga, false) => 150,
        (EnemyKind::BossGalaga, true) => 400,
        (EnemyKind::GalaxianFlagship, false) => 220,
        (EnemyKind::GalaxianFlagship, true) => 500,
        (EnemyKind::CapturedFighter, false) => 200,
        (EnemyKind::CapturedFighter, true) => 450,
    }
}

pub fn resolve_captured_ship_outcome(
    mode: EnemyMode,
    carrying_player: bool,
) -> Option<CapturedShipOutcome> {
    if !carrying_player {
        return None;
    }

    if mode == EnemyMode::Formed {
        Some(CapturedShipOutcome::BetrayLater)
    } else {
        Some(CapturedShipOutcome::RescueDual)
    }
}

pub fn stage_is_cleared(spawn_finished: bool, enemies_remaining: usize) -> bool {
    spawn_finished && enemies_remaining == 0
}

pub fn can_enemy_fire(stage_type: StageType) -> bool {
    stage_type == StageType::Normal
}

pub fn difficulty_for_stage(stage: u32) -> DifficultyTuning {
    let s = stage.saturating_sub(1) as f32;
    DifficultyTuning {
        dive_interval: (2.2 - s * 0.08).max(0.75),
        max_divers: (1 + (stage / 3)).min(6) as usize,
        enemy_fire_interval: (1.0 - s * 0.03).max(0.28),
        enemy_bullet_speed: (190.0 + s * 12.0).min(360.0),
        dive_speed_multiplier: (1.0 + s * 0.05).min(2.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_type_schedule_matches_spec() {
        assert_eq!(stage_type_for(1), StageType::Normal);
        assert_eq!(stage_type_for(2), StageType::Normal);
        assert_eq!(stage_type_for(3), StageType::Challenge);
        assert_eq!(stage_type_for(7), StageType::Challenge);
        assert_eq!(stage_type_for(11), StageType::Challenge);
        assert_eq!(stage_type_for(15), StageType::Challenge);
        assert_eq!(stage_type_for(16), StageType::Normal);
    }

    #[test]
    fn boss_hp_and_non_boss_hp_are_correct() {
        assert_eq!(enemy_base_hp(EnemyKind::BossGalaga), 2);
        assert_eq!(enemy_base_hp(EnemyKind::Bee), 1);
        assert_eq!(enemy_base_hp(EnemyKind::Butterfly), 1);
        assert_eq!(enemy_base_hp(EnemyKind::GalaxianFlagship), 1);
    }

    #[test]
    fn capture_resolution_matches_rescue_and_betrayal_rules() {
        assert_eq!(
            resolve_captured_ship_outcome(EnemyMode::Diving, true),
            Some(CapturedShipOutcome::RescueDual)
        );
        assert_eq!(
            resolve_captured_ship_outcome(EnemyMode::Capturing, true),
            Some(CapturedShipOutcome::RescueDual)
        );
        assert_eq!(
            resolve_captured_ship_outcome(EnemyMode::Formed, true),
            Some(CapturedShipOutcome::BetrayLater)
        );
        assert_eq!(
            resolve_captured_ship_outcome(EnemyMode::Formed, false),
            None
        );
    }

    #[test]
    fn challenge_stage_disables_enemy_fire() {
        assert!(can_enemy_fire(StageType::Normal));
        assert!(!can_enemy_fire(StageType::Challenge));
    }

    #[test]
    fn difficulty_increases_and_clamps() {
        let d1 = difficulty_for_stage(1);
        let d10 = difficulty_for_stage(10);
        let d40 = difficulty_for_stage(40);

        assert!(d10.enemy_bullet_speed >= d1.enemy_bullet_speed);
        assert!(d10.max_divers >= d1.max_divers);
        assert!(d10.dive_interval <= d1.dive_interval);

        assert!(d40.dive_interval >= 0.75);
        assert!(d40.enemy_fire_interval >= 0.28);
        assert!(d40.enemy_bullet_speed <= 360.0);
        assert!(d40.dive_speed_multiplier <= 2.0);
    }
}
