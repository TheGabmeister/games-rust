use crate::game::entities::{
    BUNKER_COLS, BUNKER_ROWS, Bullet, BulletOwner, Bunker, INVADER_COLS, INVADER_ROWS, Invader,
    MysteryShip,
};
use crate::game::state::{
    BULLET_HEIGHT, BULLET_WIDTH, DEFEAT_LINE, Game, GameState, INVADER_BULLET_SPEED,
    INVADER_DROP_DISTANCE, INVADER_STEP_X, LIFE_LOST_DELAY, MAX_INVADER_BULLETS, MYSTERY_HEIGHT,
    MYSTERY_SPEED, MYSTERY_WIDTH, MYSTERY_Y, PLAYER_BULLET_SPEED, PLAYFIELD_PADDING, SCREEN_HEIGHT,
    SCREEN_WIDTH, next_invader_shot_cooldown, next_mystery_spawn_delay, random_bonus_score,
    spawn_bunkers, wave_spawn_layout,
};
use macroquad::{prelude::*, rand::gen_range};

impl Game {
    pub fn update_fixed(&mut self, dt: f32) {
        match self.state {
            GameState::Start => {
                if is_key_pressed(KeyCode::Enter) {
                    self.state = GameState::Playing;
                }
            }
            GameState::Playing => {
                self.update_player(dt);
                self.update_invader_swarm(dt);
                if matches!(self.state, GameState::GameOver) {
                    return;
                }
                self.update_invader_fire(dt);
                self.update_player_bullet(dt);
                self.update_invader_bullets(dt);
                self.update_mystery_ship(dt);
            }
            GameState::LifeLost { mut timer } => {
                timer -= dt;
                if timer <= 0.0 {
                    self.state = GameState::Playing;
                } else {
                    self.state = GameState::LifeLost { timer };
                }
            }
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Enter) {
                    self.reset_run();
                    self.state = GameState::Playing;
                }
            }
        }
    }

    fn update_player(&mut self, dt: f32) {
        let mut movement = 0.0;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            movement -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            movement += 1.0;
        }

        self.player.rect.x += movement * self.player.speed * dt;
        self.player.rect.x = self.player.rect.x.clamp(
            PLAYFIELD_PADDING,
            SCREEN_WIDTH - PLAYFIELD_PADDING - self.player.rect.w,
        );

        if is_key_pressed(KeyCode::Space) && self.player_bullet.is_none() {
            let bullet_x = self.player.rect.x + self.player.rect.w * 0.5 - BULLET_WIDTH * 0.5;
            let bullet_y = self.player.rect.y - BULLET_HEIGHT;
            self.player_bullet = Some(Bullet::new(
                bullet_x,
                bullet_y,
                BULLET_WIDTH,
                BULLET_HEIGHT,
                vec2(0.0, PLAYER_BULLET_SPEED),
                BulletOwner::Player,
            ));
        }
    }

    fn update_invader_swarm(&mut self, dt: f32) {
        if self.invaders.is_empty() {
            return;
        }

        self.swarm_timer += dt;
        if self.swarm_timer < invader_move_interval(self.invaders.len()) {
            return;
        }
        self.swarm_timer = 0.0;

        let (new_direction, drop_distance) = invader_edge_step(
            &self.invaders,
            self.swarm_direction,
            PLAYFIELD_PADDING,
            SCREEN_WIDTH - PLAYFIELD_PADDING,
        );

        self.swarm_direction = new_direction;
        if drop_distance > 0.0 {
            for invader in &mut self.invaders {
                invader.rect.y += drop_distance;
            }
        } else {
            for invader in &mut self.invaders {
                invader.rect.x += INVADER_STEP_X * self.swarm_direction;
            }
        }

        if invaders_reached_defeat_line(&self.invaders, DEFEAT_LINE) {
            self.state = GameState::GameOver;
        }
    }

    fn update_invader_fire(&mut self, dt: f32) {
        if self.invader_bullets.len() >= MAX_INVADER_BULLETS || self.invaders.is_empty() {
            return;
        }

        self.invader_shot_timer += dt;
        if self.invader_shot_timer < self.invader_shot_cooldown {
            return;
        }

        self.invader_shot_timer = 0.0;
        self.invader_shot_cooldown = next_invader_shot_cooldown(self.invaders.len());

        let shooters = bottom_shooter_indices(&self.invaders);
        if shooters.is_empty() {
            return;
        }

        let picked = shooters[gen_range(0, shooters.len() as i32) as usize];
        let invader = self.invaders[picked];
        let bullet_x = invader.rect.x + invader.rect.w * 0.5 - BULLET_WIDTH * 0.5;
        let bullet_y = invader.rect.y + invader.rect.h;

        self.invader_bullets.push(Bullet::new(
            bullet_x,
            bullet_y,
            BULLET_WIDTH,
            BULLET_HEIGHT,
            vec2(0.0, INVADER_BULLET_SPEED),
            BulletOwner::Invader,
        ));
    }

    fn update_player_bullet(&mut self, dt: f32) {
        let Some(mut bullet) = self.player_bullet.take() else {
            return;
        };

        bullet.rect.y += bullet.velocity.y * dt;
        if bullet.rect.y + bullet.rect.h < 0.0 {
            return;
        }

        if let Some(ship) = self.mystery_ship
            && bullet.rect.overlaps(&ship.rect)
        {
            self.score += ship.score_value;
            self.mystery_ship = None;
            self.mystery_spawn_timer = next_mystery_spawn_delay();
            return;
        }

        if let Some(hit_idx) = self
            .invaders
            .iter()
            .position(|invader| bullet.rect.overlaps(&invader.rect))
        {
            let invader = self.invaders.swap_remove(hit_idx);
            self.score += invader.score_value;

            if self.invaders.is_empty() {
                self.wave += 1;
                self.spawn_wave();
            }
            return;
        }

        for bunker in &mut self.bunkers {
            if apply_bullet_to_bunker(bunker, &bullet) {
                return;
            }
        }

        self.player_bullet = Some(bullet);
    }

    fn update_invader_bullets(&mut self, dt: f32) {
        let mut index = 0;
        while index < self.invader_bullets.len() {
            self.invader_bullets[index].rect.y += self.invader_bullets[index].velocity.y * dt;

            if self.invader_bullets[index].rect.y > SCREEN_HEIGHT {
                self.invader_bullets.swap_remove(index);
                continue;
            }

            let bullet_rect = self.invader_bullets[index].rect;
            if bullet_rect.overlaps(&self.player.rect) {
                self.register_player_hit();
                return;
            }

            let bullet = self.invader_bullets[index];
            let mut removed = false;
            for bunker in &mut self.bunkers {
                if apply_bullet_to_bunker(bunker, &bullet) {
                    removed = true;
                    break;
                }
            }

            if removed {
                self.invader_bullets.swap_remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn update_mystery_ship(&mut self, dt: f32) {
        if let Some(ship) = &mut self.mystery_ship {
            ship.rect.x += ship.direction * ship.speed * dt;
            let off_screen = (ship.direction > 0.0 && ship.rect.x > SCREEN_WIDTH)
                || (ship.direction < 0.0 && ship.rect.x + ship.rect.w < 0.0);
            if off_screen {
                self.mystery_ship = None;
                self.mystery_spawn_timer = next_mystery_spawn_delay();
            }
        }

        if self.mystery_ship.is_none() {
            self.mystery_spawn_timer -= dt;
            if self.mystery_spawn_timer <= 0.0 {
                self.spawn_mystery_ship();
            }
        }
    }

    fn spawn_mystery_ship(&mut self) {
        let direction = if gen_range(0, 2) == 0 { 1.0 } else { -1.0 };
        let spawn_x = if direction > 0.0 {
            -MYSTERY_WIDTH - 2.0
        } else {
            SCREEN_WIDTH + 2.0
        };

        self.mystery_ship = Some(MysteryShip::new(
            spawn_x,
            MYSTERY_Y,
            MYSTERY_WIDTH,
            MYSTERY_HEIGHT,
            direction,
            MYSTERY_SPEED,
            random_bonus_score(),
        ));
        self.mystery_spawn_timer = next_mystery_spawn_delay();
    }

    fn spawn_wave(&mut self) {
        self.invaders = wave_spawn_layout(self.wave);
        self.swarm_direction = 1.0;
        self.swarm_timer = 0.0;
        self.invader_shot_timer = 0.0;
        self.invader_shot_cooldown = next_invader_shot_cooldown(self.invaders.len());
        self.player_bullet = None;
        self.invader_bullets.clear();
        self.mystery_ship = None;
        self.mystery_spawn_timer = next_mystery_spawn_delay();
    }

    fn register_player_hit(&mut self) {
        if !matches!(self.state, GameState::Playing) {
            return;
        }

        self.lives -= 1;
        self.player_bullet = None;
        self.invader_bullets.clear();
        self.reset_player_position();

        if self.lives <= 0 {
            self.state = GameState::GameOver;
        } else {
            self.state = GameState::LifeLost {
                timer: LIFE_LOST_DELAY,
            };
        }
    }

    fn reset_run(&mut self) {
        self.score = 0;
        self.lives = 3;
        self.wave = 1;
        self.state = GameState::Start;
        self.bunkers = spawn_bunkers();
        self.spawn_wave();
        self.reset_player_position();
    }
}

fn invader_move_interval(alive_count: usize) -> f32 {
    let clamped_alive = alive_count.clamp(1, INVADER_ROWS * INVADER_COLS) as f32;
    let ratio = (clamped_alive - 1.0) / ((INVADER_ROWS * INVADER_COLS - 1) as f32);
    0.08 + ratio * 0.62
}

fn invader_edge_step(
    invaders: &[Invader],
    direction: f32,
    left_bound: f32,
    right_bound: f32,
) -> (f32, f32) {
    if invaders.is_empty() {
        return (direction, 0.0);
    }

    let min_x = invaders
        .iter()
        .map(|invader| invader.rect.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = invaders
        .iter()
        .map(|invader| invader.rect.x + invader.rect.w)
        .fold(f32::NEG_INFINITY, f32::max);

    if direction > 0.0 && max_x + INVADER_STEP_X >= right_bound {
        (-1.0, INVADER_DROP_DISTANCE)
    } else if direction < 0.0 && min_x - INVADER_STEP_X <= left_bound {
        (1.0, INVADER_DROP_DISTANCE)
    } else {
        (direction, 0.0)
    }
}

fn bottom_shooter_indices(invaders: &[Invader]) -> Vec<usize> {
    let mut bottom_by_col: [Option<(usize, f32)>; INVADER_COLS] = [None; INVADER_COLS];

    for (idx, invader) in invaders.iter().enumerate() {
        if invader.col >= INVADER_COLS {
            continue;
        }

        let bottom_y = invader.rect.y + invader.rect.h;
        match bottom_by_col[invader.col] {
            Some((_, existing_y)) if existing_y >= bottom_y => {}
            _ => bottom_by_col[invader.col] = Some((idx, bottom_y)),
        }
    }

    bottom_by_col
        .iter()
        .filter_map(|entry| entry.map(|(idx, _)| idx))
        .collect()
}

fn damage_bunker_cells(
    cells: &mut [[bool; BUNKER_COLS]; BUNKER_ROWS],
    hit_col: i32,
    hit_row: i32,
    owner: BulletOwner,
) -> usize {
    let row_offsets: [i32; 2] = match owner {
        BulletOwner::Player => [0, 1],
        BulletOwner::Invader => [-1, 0],
    };

    let mut removed = 0;
    for row_offset in row_offsets {
        for col_offset in -1..=1 {
            let row = hit_row + row_offset;
            let col = hit_col + col_offset;
            if row < 0 || row >= BUNKER_ROWS as i32 || col < 0 || col >= BUNKER_COLS as i32 {
                continue;
            }

            let cell = &mut cells[row as usize][col as usize];
            if *cell {
                *cell = false;
                removed += 1;
            }
        }
    }

    removed
}

fn invaders_reached_defeat_line(invaders: &[Invader], defeat_line: f32) -> bool {
    invaders
        .iter()
        .any(|invader| invader.rect.y + invader.rect.h >= defeat_line)
}

fn apply_bullet_to_bunker(bunker: &mut Bunker, bullet: &Bullet) -> bool {
    if !bunker.bounds().overlaps(&bullet.rect) {
        return false;
    }

    let impact_x = bullet.rect.x + bullet.rect.w * 0.5 - bunker.position.x;
    let impact_y = match bullet.owner {
        BulletOwner::Player => bullet.rect.y - bunker.position.y,
        BulletOwner::Invader => bullet.rect.y + bullet.rect.h - bunker.position.y,
    };

    if impact_x < 0.0 || impact_x >= bunker.width() || impact_y < 0.0 || impact_y >= bunker.height()
    {
        return false;
    }

    let hit_col = (impact_x / bunker.cell_size).floor() as i32;
    let hit_row = (impact_y / bunker.cell_size).floor() as i32;
    damage_bunker_cells(&mut bunker.cells, hit_col, hit_row, bullet.owner) > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::state::{
        DEFEAT_LINE, INVADER_HEIGHT, INVADER_WIDTH, PLAYFIELD_PADDING, SCREEN_WIDTH,
        wave_spawn_layout,
    };

    #[test]
    fn wave_spawn_layout_has_expected_count_and_scores() {
        let invaders = wave_spawn_layout(1);
        assert_eq!(invaders.len(), INVADER_ROWS * INVADER_COLS);

        let mut top = 0;
        let mut middle = 0;
        let mut bottom = 0;

        for invader in invaders {
            match invader.score_value {
                30 => top += 1,
                20 => middle += 1,
                10 => bottom += 1,
                _ => panic!("unexpected score value"),
            }
        }

        assert_eq!(top, INVADER_COLS);
        assert_eq!(middle, INVADER_COLS * 2);
        assert_eq!(bottom, INVADER_COLS * 2);
    }

    #[test]
    fn invader_edge_step_reverses_and_drops_at_right_edge() {
        let invaders = vec![Invader::new(
            900.0,
            120.0,
            INVADER_WIDTH,
            INVADER_HEIGHT,
            0,
            0,
            30,
        )];
        let (dir, drop) = invader_edge_step(
            &invaders,
            1.0,
            PLAYFIELD_PADDING,
            SCREEN_WIDTH - PLAYFIELD_PADDING,
        );
        assert_eq!(dir, -1.0);
        assert!(drop > 0.0);
    }

    #[test]
    fn invader_move_interval_speeds_up_as_invaders_die() {
        let full = invader_move_interval(INVADER_ROWS * INVADER_COLS);
        let mid = invader_move_interval(20);
        let low = invader_move_interval(1);

        assert!(full > mid);
        assert!(mid > low);
    }

    #[test]
    fn bottom_shooter_selection_picks_lowest_invader_per_column() {
        let invaders = vec![
            Invader::new(100.0, 120.0, INVADER_WIDTH, INVADER_HEIGHT, 0, 0, 30),
            Invader::new(100.0, 180.0, INVADER_WIDTH, INVADER_HEIGHT, 4, 0, 10),
            Invader::new(180.0, 140.0, INVADER_WIDTH, INVADER_HEIGHT, 1, 1, 20),
        ];

        let indices = bottom_shooter_indices(&invaders);
        assert_eq!(indices, vec![1, 2]);
    }

    #[test]
    fn bunker_damage_mutates_mask() {
        let mut bunker = Bunker::new(vec2(0.0, 0.0), crate::game::state::BUNKER_CELL_SIZE);
        let removed = damage_bunker_cells(&mut bunker.cells, 4, 2, BulletOwner::Invader);

        assert!(removed > 0);
        assert!(!bunker.cells[2][4]);
    }

    #[test]
    fn game_over_conditions_are_enforced() {
        let mut game = Game::new();
        game.state = GameState::Playing;
        game.lives = 1;
        game.register_player_hit();
        assert!(matches!(game.state, GameState::GameOver));

        let invaders = vec![Invader::new(
            100.0,
            DEFEAT_LINE - INVADER_HEIGHT + 1.0,
            INVADER_WIDTH,
            INVADER_HEIGHT,
            0,
            0,
            30,
        )];
        assert!(invaders_reached_defeat_line(&invaders, DEFEAT_LINE));
    }
}
