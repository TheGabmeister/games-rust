use ::rand::{Rng, SeedableRng, rngs::SmallRng};
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub grid_w: i32,
    pub grid_h: i32,
    pub player_rows: i32,
    pub cell_size: f32,
    pub fixed_dt: f32,
    pub player_speed: f32,
    pub fire_cooldown: f32,
    pub projectile_speed: f32,
    pub centipede_step_interval: f32,
    pub detached_head_step_interval: f32,
    pub flea_speed: f32,
    pub flea_spawn_threshold: i32,
    pub flea_spawn_cooldown: f32,
    pub flea_trail_chance: f32,
    pub spider_spawn_cooldown: f32,
    pub spider_speed_x: f32,
    pub spider_speed_y: f32,
    pub scorpion_spawn_cooldown: f32,
    pub scorpion_speed: f32,
    pub side_head_spawn_interval: f32,
    pub initial_lives: u32,
    pub initial_centipede_length: usize,
    pub score_head: u32,
    pub score_body: u32,
    pub score_mushroom_hit: u32,
    pub score_flea: u32,
    pub score_scorpion: u32,
    pub score_spider_far: u32,
    pub score_spider_mid: u32,
    pub score_spider_near: u32,
    pub extra_life_score: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            grid_w: 30,
            grid_h: 32,
            player_rows: 8,
            cell_size: 16.0,
            fixed_dt: 1.0 / 60.0,
            player_speed: 11.0,
            fire_cooldown: 0.16,
            projectile_speed: 30.0,
            centipede_step_interval: 0.12,
            detached_head_step_interval: 0.09,
            flea_speed: 12.0,
            flea_spawn_threshold: 5,
            flea_spawn_cooldown: 1.5,
            flea_trail_chance: 0.35,
            spider_spawn_cooldown: 8.0,
            spider_speed_x: 8.0,
            spider_speed_y: 4.0,
            scorpion_spawn_cooldown: 11.0,
            scorpion_speed: 11.0,
            side_head_spawn_interval: 4.0,
            initial_lives: 3,
            initial_centipede_length: 12,
            score_head: 100,
            score_body: 10,
            score_mushroom_hit: 1,
            score_flea: 200,
            score_scorpion: 1000,
            score_spider_far: 300,
            score_spider_mid: 600,
            score_spider_near: 900,
            extra_life_score: 12_000,
        }
    }
}

impl GameConfig {
    pub fn logical_width_px(&self) -> f32 {
        self.grid_w as f32 * self.cell_size
    }

    pub fn logical_height_px(&self) -> f32 {
        self.grid_h as f32 * self.cell_size
    }

    pub fn player_area_start_row(&self) -> i32 {
        self.grid_h - self.player_rows
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CommandInput {
    pub move_axis: Vec2,
    pub fire: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    CentipedeHead,
    CentipedeBody,
    DetachedHead,
    Flea,
    Spider,
    Scorpion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    Shot,
    Hit,
    EnemyKilled(EnemyKind),
    PlayerDied,
    RoundCleared(u32),
    ExtraLife,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CentipedeMode {
    Normal,
    PoisonDive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub pos: IVec2,
    pub is_head: bool,
}

#[derive(Debug, Clone)]
pub struct CentipedeChain {
    pub segments: Vec<Segment>,
    pub dir: i32,
    pub mode: CentipedeMode,
    pub entered_player_area: bool,
    pub step_interval: f32,
    pub step_timer: f32,
    pub origin_main: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MushroomCell {
    pub hp: u8,
    pub poisoned: bool,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub pos: Vec2,
    pub fire_cooldown: f32,
}

#[derive(Debug, Clone)]
pub struct Projectile {
    pub pos: Vec2,
}

#[derive(Debug, Clone)]
pub struct Flea {
    pub pos: Vec2,
    pub hp: u8,
    pub last_row: i32,
}

#[derive(Debug, Clone)]
pub struct Spider {
    pub pos: Vec2,
    pub vel: Vec2,
    pub zigzag_timer: f32,
}

#[derive(Debug, Clone)]
pub struct Scorpion {
    pub pos: Vec2,
    pub dir: i32,
}

#[derive(Debug, Clone)]
pub struct DetachedHead {
    pub segment: Segment,
    pub dir: i32,
    pub mode: CentipedeMode,
    pub entered_player_area: bool,
    pub step_interval: f32,
    pub step_timer: f32,
    pub origin_main: bool,
}

#[derive(Debug, Clone)]
pub enum Enemy {
    Flea(Flea),
    Spider(Spider),
    Scorpion(Scorpion),
    DetachedHead(DetachedHead),
}

#[derive(Debug, Clone)]
pub struct RoundDirector {
    initial_length: usize,
    cycle_index: usize,
}

impl RoundDirector {
    pub fn new(initial_length: usize) -> Self {
        Self {
            initial_length: initial_length.max(1),
            cycle_index: 0,
        }
    }

    pub fn current_main_length(&self) -> usize {
        self.initial_length - self.cycle_index
    }

    pub fn current_bonus_heads(&self) -> usize {
        self.cycle_index
    }

    pub fn advance(&mut self) {
        self.cycle_index = (self.cycle_index + 1) % self.initial_length;
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub config: GameConfig,
    pub player: Player,
    pub projectiles: Vec<Projectile>,
    pub mushrooms: Vec<Option<MushroomCell>>,
    pub centipede_chains: Vec<CentipedeChain>,
    pub enemies: Vec<Enemy>,
    pub score: u32,
    pub lives: u32,
    pub round: u32,
    pub round_director: RoundDirector,
    game_over: bool,
    bottom_phase_active: bool,
    events: Vec<GameEvent>,
    rng: SmallRng,
    next_extra_life_score: u32,
    flea_spawn_timer: f32,
    spider_spawn_timer: f32,
    scorpion_spawn_timer: f32,
    side_head_spawn_timer: f32,
}

impl World {
    pub fn new(config: GameConfig, seed: u64) -> Self {
        let mut world = Self {
            player: Player {
                pos: vec2(
                    (config.grid_w as f32 - 1.0) * 0.5,
                    config.grid_h as f32 - 1.5,
                ),
                fire_cooldown: 0.0,
            },
            projectiles: Vec::new(),
            mushrooms: vec![None; (config.grid_w * config.grid_h) as usize],
            centipede_chains: Vec::new(),
            enemies: Vec::new(),
            score: 0,
            lives: config.initial_lives,
            round: 1,
            round_director: RoundDirector::new(config.initial_centipede_length),
            game_over: false,
            bottom_phase_active: false,
            events: Vec::new(),
            rng: SmallRng::seed_from_u64(seed),
            next_extra_life_score: config.extra_life_score,
            flea_spawn_timer: config.flea_spawn_cooldown,
            spider_spawn_timer: config.spider_spawn_cooldown,
            scorpion_spawn_timer: config.scorpion_spawn_cooldown,
            side_head_spawn_timer: config.side_head_spawn_interval,
            config,
        };

        world.seed_initial_mushrooms();
        world.spawn_round_entities();
        world
    }

    pub fn emit_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn mushroom_cell(&self, cell: IVec2) -> Option<MushroomCell> {
        self.cell_index(cell).and_then(|idx| self.mushrooms[idx])
    }

    pub fn update(&mut self, dt: f32, input: CommandInput) {
        if self.game_over {
            return;
        }

        self.update_player(dt, input);
        self.update_projectiles(dt);
        self.update_centipede_chains(dt);
        self.update_enemies(dt);
        self.resolve_projectile_collisions();
        self.cleanup_entities();
        self.check_player_collisions();
        self.handle_spawns(dt);
        self.check_round_clear();
    }

    fn update_player(&mut self, dt: f32, input: CommandInput) {
        self.player.fire_cooldown = (self.player.fire_cooldown - dt).max(0.0);

        self.player.pos += input.move_axis * self.config.player_speed * dt;
        self.player.pos.x = self
            .player
            .pos
            .x
            .clamp(0.0, self.config.grid_w as f32 - 1.0);
        self.player.pos.y = self.player.pos.y.clamp(
            self.config.player_area_start_row() as f32,
            self.config.grid_h as f32 - 1.0,
        );

        if input.fire && self.player.fire_cooldown <= 0.0 {
            self.projectiles.push(Projectile {
                pos: self.player.pos + vec2(0.0, -0.7),
            });
            self.player.fire_cooldown = self.config.fire_cooldown;
            self.events.push(GameEvent::Shot);
        }
    }

    fn update_projectiles(&mut self, dt: f32) {
        for projectile in &mut self.projectiles {
            projectile.pos.y -= self.config.projectile_speed * dt;
        }
        self.projectiles
            .retain(|projectile| projectile.pos.y >= -1.0);
    }

    fn update_centipede_chains(&mut self, dt: f32) {
        for idx in 0..self.centipede_chains.len() {
            let interval = self.centipede_chains[idx].step_interval;
            self.centipede_chains[idx].step_timer += dt;
            while self.centipede_chains[idx].step_timer >= interval {
                self.centipede_chains[idx].step_timer -= interval;
                self.step_chain_once(idx);
            }
        }
    }

    fn step_chain_once(&mut self, idx: usize) {
        if idx >= self.centipede_chains.len() || self.centipede_chains[idx].segments.is_empty() {
            return;
        }

        let head = self.centipede_chains[idx].segments[0].pos;
        let result = compute_head_step(
            &self.config,
            &self.mushrooms,
            head,
            self.centipede_chains[idx].dir,
            self.centipede_chains[idx].mode,
            self.centipede_chains[idx].entered_player_area,
        );

        self.centipede_chains[idx].dir = result.dir;
        self.centipede_chains[idx].mode = result.mode;
        self.centipede_chains[idx].entered_player_area = result.entered_player_area;
        shift_segments(&mut self.centipede_chains[idx].segments, result.next_head);

        if self.centipede_chains[idx].origin_main && result.entered_player_area {
            self.bottom_phase_active = true;
        }
    }

    fn update_enemies(&mut self, dt: f32) {
        let enemies = std::mem::take(&mut self.enemies);
        let mut next_enemies = Vec::with_capacity(enemies.len());
        let player_area_mushrooms = self.count_player_area_mushrooms();

        for mut enemy in enemies {
            let keep = match &mut enemy {
                Enemy::Flea(flea) => {
                    flea.pos.y += self.config.flea_speed * dt;
                    let row = flea.pos.y.round() as i32;
                    if row != flea.last_row {
                        flea.last_row = row;
                        if player_area_mushrooms < self.config.flea_spawn_threshold
                            && self.rng.gen_bool(self.config.flea_trail_chance as f64)
                        {
                            let spawn_cell = ivec2(flea.pos.x.round() as i32, row);
                            self.spawn_mushroom(spawn_cell, false, 4);
                        }
                    }
                    flea.pos.y < self.config.grid_h as f32 + 1.0
                }
                Enemy::Spider(spider) => {
                    spider.pos += spider.vel * dt;
                    spider.zigzag_timer -= dt;
                    if spider.zigzag_timer <= 0.0 {
                        spider.zigzag_timer = self.rng.gen_range(0.2..0.8);
                        spider.vel.y = -spider.vel.y;
                    }

                    let min_y = self.config.player_area_start_row() as f32;
                    let max_y = self.config.grid_h as f32 - 1.0;
                    if spider.pos.y < min_y {
                        spider.pos.y = min_y;
                        spider.vel.y = spider.vel.y.abs();
                    } else if spider.pos.y > max_y {
                        spider.pos.y = max_y;
                        spider.vel.y = -spider.vel.y.abs();
                    }

                    if let Some(cell) = pos_to_cell_in_bounds(&self.config, spider.pos) {
                        if let Some(idx) = self.cell_index(cell) {
                            self.mushrooms[idx] = None;
                        }
                    }

                    spider.pos.x > -2.0 && spider.pos.x < self.config.grid_w as f32 + 2.0
                }
                Enemy::Scorpion(scorpion) => {
                    scorpion.pos.x += scorpion.dir as f32 * self.config.scorpion_speed * dt;
                    if let Some(cell) = pos_to_cell_in_bounds(&self.config, scorpion.pos) {
                        if let Some(idx) = self.cell_index(cell) {
                            if let Some(mushroom) = self.mushrooms[idx].as_mut() {
                                mushroom.poisoned = true;
                            }
                        }
                    }
                    scorpion.pos.x > -2.0 && scorpion.pos.x < self.config.grid_w as f32 + 2.0
                }
                Enemy::DetachedHead(head) => {
                    head.step_timer += dt;
                    while head.step_timer >= head.step_interval {
                        head.step_timer -= head.step_interval;
                        let result = compute_head_step(
                            &self.config,
                            &self.mushrooms,
                            head.segment.pos,
                            head.dir,
                            head.mode,
                            head.entered_player_area,
                        );
                        head.segment.pos = result.next_head;
                        head.segment.is_head = true;
                        head.dir = result.dir;
                        head.mode = result.mode;
                        head.entered_player_area = result.entered_player_area;
                        if head.origin_main && result.entered_player_area {
                            self.bottom_phase_active = true;
                        }
                    }
                    true
                }
            };

            if keep {
                next_enemies.push(enemy);
            }
        }

        self.enemies = next_enemies;
    }

    fn resolve_projectile_collisions(&mut self) {
        let mut kept = Vec::with_capacity(self.projectiles.len());
        let player_pos = self.player.pos;
        let projectiles = std::mem::take(&mut self.projectiles);

        for projectile in projectiles {
            if let Some(cell) = pos_to_cell_in_bounds(&self.config, projectile.pos) {
                if self.projectile_hits_centipede(cell)
                    || self.projectile_hits_enemy(cell, player_pos)
                    || self.projectile_hits_mushroom(cell)
                {
                    self.events.push(GameEvent::Hit);
                    continue;
                }
            }
            kept.push(projectile);
        }

        self.projectiles = kept;
    }

    fn projectile_hits_centipede(&mut self, cell: IVec2) -> bool {
        for chain_idx in 0..self.centipede_chains.len() {
            for seg_idx in 0..self.centipede_chains[chain_idx].segments.len() {
                if self.centipede_chains[chain_idx].segments[seg_idx].pos == cell {
                    self.apply_chain_hit(chain_idx, seg_idx);
                    return true;
                }
            }
        }
        false
    }

    fn apply_chain_hit(&mut self, chain_idx: usize, seg_idx: usize) {
        if chain_idx >= self.centipede_chains.len() {
            return;
        }
        if seg_idx >= self.centipede_chains[chain_idx].segments.len() {
            return;
        }

        let hit_pos = self.centipede_chains[chain_idx].segments[seg_idx].pos;
        let was_head = seg_idx == 0;
        self.spawn_mushroom(hit_pos, false, 4);

        if was_head {
            self.add_score(self.config.score_head);
            self.events
                .push(GameEvent::EnemyKilled(EnemyKind::CentipedeHead));
        } else {
            self.add_score(self.config.score_body);
            self.events
                .push(GameEvent::EnemyKilled(EnemyKind::CentipedeBody));
        }

        let mut rear_chain: Option<CentipedeChain> = None;
        {
            let chain = &mut self.centipede_chains[chain_idx];
            if seg_idx > 0 && seg_idx + 1 < chain.segments.len() {
                let mut rear_segments = chain.segments.split_off(seg_idx + 1);
                reset_head_flags(&mut rear_segments);
                rear_chain = Some(CentipedeChain {
                    segments: rear_segments,
                    dir: chain.dir,
                    mode: chain.mode,
                    entered_player_area: chain.entered_player_area,
                    step_interval: chain.step_interval,
                    step_timer: chain.step_timer,
                    origin_main: chain.origin_main,
                });
            }
            chain.segments.remove(seg_idx);
            reset_head_flags(&mut chain.segments);
        }

        if let Some(chain) = rear_chain {
            self.centipede_chains.push(chain);
        }
    }

    fn projectile_hits_enemy(&mut self, cell: IVec2, player_pos: Vec2) -> bool {
        let mut index = 0usize;
        while index < self.enemies.len() {
            let is_hit = match &mut self.enemies[index] {
                Enemy::Flea(flea) => {
                    if pos_to_cell_in_bounds(&self.config, flea.pos) == Some(cell) {
                        if flea.hp > 0 {
                            flea.hp -= 1;
                        }
                        if flea.hp == 0 {
                            self.add_score(self.config.score_flea);
                            self.events.push(GameEvent::EnemyKilled(EnemyKind::Flea));
                            self.enemies.remove(index);
                        }
                        true
                    } else {
                        false
                    }
                }
                Enemy::Spider(spider) => {
                    if pos_to_cell_in_bounds(&self.config, spider.pos) == Some(cell) {
                        let points = spider_score_for_range(&self.config, player_pos, spider.pos);
                        self.add_score(points);
                        self.events.push(GameEvent::EnemyKilled(EnemyKind::Spider));
                        self.enemies.remove(index);
                        true
                    } else {
                        false
                    }
                }
                Enemy::Scorpion(scorpion) => {
                    if pos_to_cell_in_bounds(&self.config, scorpion.pos) == Some(cell) {
                        self.add_score(self.config.score_scorpion);
                        self.events
                            .push(GameEvent::EnemyKilled(EnemyKind::Scorpion));
                        self.enemies.remove(index);
                        true
                    } else {
                        false
                    }
                }
                Enemy::DetachedHead(head) => {
                    if head.segment.pos == cell {
                        self.add_score(self.config.score_head);
                        self.events
                            .push(GameEvent::EnemyKilled(EnemyKind::DetachedHead));
                        self.enemies.remove(index);
                        true
                    } else {
                        false
                    }
                }
            };

            if is_hit {
                return true;
            }
            index += 1;
        }
        false
    }

    fn projectile_hits_mushroom(&mut self, cell: IVec2) -> bool {
        if let Some(idx) = self.cell_index(cell) {
            let mut was_hit = false;
            let mut destroyed = false;
            if let Some(mushroom) = self.mushrooms[idx].as_mut() {
                was_hit = true;
                mushroom.hp = mushroom.hp.saturating_sub(1);
                destroyed = mushroom.hp == 0;
            }
            if was_hit {
                self.add_score(self.config.score_mushroom_hit);
                if destroyed {
                    self.mushrooms[idx] = None;
                }
                return true;
            }
        }
        false
    }

    fn cleanup_entities(&mut self) {
        self.centipede_chains
            .retain(|chain| !chain.segments.is_empty());
    }

    fn check_player_collisions(&mut self) {
        if self.game_over {
            return;
        }

        let Some(player_cell) = self.clamp_to_cell(self.player.pos) else {
            return;
        };

        for chain in &self.centipede_chains {
            if chain
                .segments
                .iter()
                .any(|segment| segment.pos == player_cell)
            {
                self.handle_player_death();
                return;
            }
        }

        for enemy in &self.enemies {
            let hit = match enemy {
                Enemy::Flea(flea) => self.clamp_to_cell(flea.pos) == Some(player_cell),
                Enemy::Spider(spider) => self.clamp_to_cell(spider.pos) == Some(player_cell),
                Enemy::Scorpion(scorpion) => self.clamp_to_cell(scorpion.pos) == Some(player_cell),
                Enemy::DetachedHead(head) => head.segment.pos == player_cell,
            };
            if hit {
                self.handle_player_death();
                return;
            }
        }
    }

    fn handle_spawns(&mut self, dt: f32) {
        self.flea_spawn_timer -= dt;
        self.spider_spawn_timer -= dt;
        self.scorpion_spawn_timer -= dt;
        self.side_head_spawn_timer -= dt;

        let has_flea = self
            .enemies
            .iter()
            .any(|enemy| matches!(enemy, Enemy::Flea(_)));
        if !has_flea
            && self.flea_spawn_timer <= 0.0
            && self.count_player_area_mushrooms() < self.config.flea_spawn_threshold
        {
            self.spawn_flea();
            self.flea_spawn_timer = self.config.flea_spawn_cooldown;
        }

        let has_spider = self
            .enemies
            .iter()
            .any(|enemy| matches!(enemy, Enemy::Spider(_)));
        if !has_spider && self.spider_spawn_timer <= 0.0 {
            self.spawn_spider();
            self.spider_spawn_timer =
                self.config.spider_spawn_cooldown + self.rng.gen_range(0.0..2.5);
        }

        let has_scorpion = self
            .enemies
            .iter()
            .any(|enemy| matches!(enemy, Enemy::Scorpion(_)));
        if !has_scorpion && self.scorpion_spawn_timer <= 0.0 {
            self.spawn_scorpion();
            self.scorpion_spawn_timer =
                self.config.scorpion_spawn_cooldown + self.rng.gen_range(0.0..2.0);
        }

        if self.bottom_phase_active
            && self.main_origin_segment_count() > 0
            && self.side_head_spawn_timer <= 0.0
        {
            self.spawn_side_detached_head();
            self.side_head_spawn_timer = self.config.side_head_spawn_interval;
        }
    }

    fn check_round_clear(&mut self) {
        let has_centipede = !self.centipede_chains.is_empty();
        let has_detached = self
            .enemies
            .iter()
            .any(|enemy| matches!(enemy, Enemy::DetachedHead(_)));

        if !has_centipede && !has_detached {
            let cleared_round = self.round;
            self.events.push(GameEvent::RoundCleared(cleared_round));
            self.round += 1;
            self.round_director.advance();
            self.spawn_round_entities();
        }
    }

    fn handle_player_death(&mut self) {
        self.events.push(GameEvent::PlayerDied);
        self.projectiles.clear();

        let mut regenerated = 0u32;
        for mushroom in self.mushrooms.iter_mut().flatten() {
            if mushroom.poisoned || mushroom.hp < 4 {
                mushroom.poisoned = false;
                mushroom.hp = 4;
                regenerated += 1;
            }
        }
        self.add_score(regenerated * 5);

        self.player.pos = vec2(
            (self.config.grid_w as f32 - 1.0) * 0.5,
            self.config.grid_h as f32 - 1.5,
        );
        self.player.fire_cooldown = 0.0;

        if self.lives > 0 {
            self.lives -= 1;
        }
        if self.lives == 0 {
            self.game_over = true;
        }
    }

    fn add_score(&mut self, amount: u32) {
        self.score = self.score.saturating_add(amount);
        while self.score >= self.next_extra_life_score {
            self.lives = self.lives.saturating_add(1);
            self.next_extra_life_score = self
                .next_extra_life_score
                .saturating_add(self.config.extra_life_score);
            self.events.push(GameEvent::ExtraLife);
        }
    }

    fn seed_initial_mushrooms(&mut self) {
        let total = ((self.config.grid_w * self.config.grid_h) as f32 * 0.12) as usize;
        for _ in 0..total {
            let cell = ivec2(
                self.rng.gen_range(0..self.config.grid_w),
                self.rng
                    .gen_range(1..self.config.player_area_start_row().max(2)),
            );
            self.spawn_mushroom(cell, false, 4);
        }
    }

    fn spawn_round_entities(&mut self) {
        self.centipede_chains.clear();
        self.enemies.clear();
        self.projectiles.clear();
        self.bottom_phase_active = false;
        self.side_head_spawn_timer = self.config.side_head_spawn_interval;
        self.flea_spawn_timer = self.config.flea_spawn_cooldown;
        self.spider_spawn_timer = self.config.spider_spawn_cooldown;
        self.scorpion_spawn_timer = self.config.scorpion_spawn_cooldown;

        let length = self.round_director.current_main_length();
        let mut segments = Vec::with_capacity(length);
        let dir = if self.rng.gen_bool(0.5) { 1 } else { -1 };
        for i in 0..length {
            let x = if dir == 1 {
                i as i32
            } else {
                self.config.grid_w - 1 - i as i32
            };
            segments.push(Segment {
                pos: ivec2(x, 0),
                is_head: i == 0,
            });
        }

        self.centipede_chains.push(CentipedeChain {
            segments,
            dir,
            mode: CentipedeMode::Normal,
            entered_player_area: false,
            step_interval: self.config.centipede_step_interval,
            step_timer: 0.0,
            origin_main: true,
        });

        for _ in 0..self.round_director.current_bonus_heads() {
            self.spawn_bonus_detached_head();
        }
    }

    fn spawn_flea(&mut self) {
        let x = self.rng.gen_range(0..self.config.grid_w) as f32;
        self.enemies.push(Enemy::Flea(Flea {
            pos: vec2(x, -1.0),
            hp: 2,
            last_row: -10,
        }));
    }

    fn spawn_spider(&mut self) {
        let from_left = self.rng.gen_bool(0.5);
        let min_row = self.config.player_area_start_row();
        let row = self.rng.gen_range(min_row..self.config.grid_h);
        let x = if from_left {
            -1.5
        } else {
            self.config.grid_w as f32 + 1.5
        };
        let vx = if from_left {
            self.config.spider_speed_x
        } else {
            -self.config.spider_speed_x
        };
        let vy = if self.rng.gen_bool(0.5) {
            self.config.spider_speed_y
        } else {
            -self.config.spider_speed_y
        };

        self.enemies.push(Enemy::Spider(Spider {
            pos: vec2(x, row as f32),
            vel: vec2(vx, vy),
            zigzag_timer: self.rng.gen_range(0.25..0.9),
        }));
    }

    fn spawn_scorpion(&mut self) {
        let from_left = self.rng.gen_bool(0.5);
        let max_row = self.config.player_area_start_row().max(4);
        let row = self.rng.gen_range(2..max_row);
        let x = if from_left {
            -1.5
        } else {
            self.config.grid_w as f32 + 1.5
        };
        let dir = if from_left { 1 } else { -1 };

        self.enemies.push(Enemy::Scorpion(Scorpion {
            pos: vec2(x, row as f32),
            dir,
        }));
    }

    fn spawn_side_detached_head(&mut self) {
        let from_left = self.rng.gen_bool(0.5);
        let row = self
            .rng
            .gen_range(self.config.player_area_start_row()..self.config.grid_h);
        let x = if from_left { 0 } else { self.config.grid_w - 1 };
        let dir = if from_left { 1 } else { -1 };
        self.enemies.push(Enemy::DetachedHead(DetachedHead {
            segment: Segment {
                pos: ivec2(x, row),
                is_head: true,
            },
            dir,
            mode: CentipedeMode::Normal,
            entered_player_area: true,
            step_interval: self.config.detached_head_step_interval,
            step_timer: 0.0,
            origin_main: false,
        }));
    }

    fn spawn_bonus_detached_head(&mut self) {
        let x = self.rng.gen_range(0..self.config.grid_w);
        let dir = if self.rng.gen_bool(0.5) { 1 } else { -1 };
        self.enemies.push(Enemy::DetachedHead(DetachedHead {
            segment: Segment {
                pos: ivec2(x, 1),
                is_head: true,
            },
            dir,
            mode: CentipedeMode::Normal,
            entered_player_area: false,
            step_interval: self.config.detached_head_step_interval,
            step_timer: 0.0,
            origin_main: false,
        }));
    }

    fn spawn_mushroom(&mut self, cell: IVec2, poisoned: bool, hp: u8) {
        if let Some(idx) = self.cell_index(cell) {
            self.mushrooms[idx] = Some(MushroomCell {
                hp: hp.clamp(1, 4),
                poisoned,
            });
        }
    }

    fn main_origin_segment_count(&self) -> usize {
        self.centipede_chains
            .iter()
            .filter(|chain| chain.origin_main)
            .map(|chain| chain.segments.len())
            .sum()
    }

    fn count_player_area_mushrooms(&self) -> i32 {
        let mut count = 0;
        for y in self.config.player_area_start_row()..self.config.grid_h {
            for x in 0..self.config.grid_w {
                if let Some(idx) = self.cell_index(ivec2(x, y)) {
                    if self.mushrooms[idx].is_some() {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn clamp_to_cell(&self, pos: Vec2) -> Option<IVec2> {
        let cell = ivec2(pos.x.round() as i32, pos.y.round() as i32);
        if self.cell_index(cell).is_some() {
            Some(cell)
        } else {
            None
        }
    }

    fn cell_index(&self, cell: IVec2) -> Option<usize> {
        if cell.x < 0 || cell.y < 0 || cell.x >= self.config.grid_w || cell.y >= self.config.grid_h
        {
            return None;
        }
        Some((cell.y * self.config.grid_w + cell.x) as usize)
    }
}

#[derive(Debug, Clone, Copy)]
struct HeadStepResult {
    next_head: IVec2,
    dir: i32,
    mode: CentipedeMode,
    entered_player_area: bool,
}

fn compute_head_step(
    config: &GameConfig,
    mushrooms: &[Option<MushroomCell>],
    head: IVec2,
    dir: i32,
    mode: CentipedeMode,
    entered_player_area: bool,
) -> HeadStepResult {
    let mut next_head = head;
    let mut next_dir = dir;
    let mut next_mode = mode;
    let mut entered = entered_player_area;
    let player_row = config.player_area_start_row();

    match mode {
        CentipedeMode::PoisonDive => {
            next_head.y += 1;
            if next_head.y >= config.grid_h - 1 {
                next_head.y = config.grid_h - 1;
                next_mode = CentipedeMode::Normal;
            }
        }
        CentipedeMode::Normal => {
            let candidate = ivec2(head.x + dir, head.y);
            let candidate_mushroom = mushroom_at(config, mushrooms, candidate);
            if candidate.x < 0 || candidate.x >= config.grid_w || candidate_mushroom.is_some() {
                next_head.y += 1;
                next_dir = -dir;
                if let Some(mushroom) = candidate_mushroom {
                    if mushroom.poisoned {
                        next_mode = CentipedeMode::PoisonDive;
                    }
                }
                if next_head.y >= config.grid_h {
                    next_head.y = if entered {
                        player_row
                    } else {
                        config.grid_h - 1
                    };
                }
            } else {
                next_head = candidate;
            }
        }
    }

    if next_head.y >= player_row {
        entered = true;
    }
    if entered {
        if next_head.y < player_row {
            next_head.y = player_row;
        }
        if next_head.y >= config.grid_h {
            next_head.y = player_row;
        }
    }

    if next_mode == CentipedeMode::Normal {
        if let Some(mushroom) = mushroom_at(config, mushrooms, next_head) {
            if mushroom.poisoned {
                next_mode = CentipedeMode::PoisonDive;
            }
        }
    }

    HeadStepResult {
        next_head,
        dir: next_dir,
        mode: next_mode,
        entered_player_area: entered,
    }
}

fn shift_segments(segments: &mut [Segment], new_head: IVec2) {
    if segments.is_empty() {
        return;
    }
    for idx in (1..segments.len()).rev() {
        segments[idx].pos = segments[idx - 1].pos;
    }
    segments[0].pos = new_head;
    reset_head_flags(segments);
}

fn reset_head_flags(segments: &mut [Segment]) {
    for (idx, segment) in segments.iter_mut().enumerate() {
        segment.is_head = idx == 0;
    }
}

fn mushroom_at(
    config: &GameConfig,
    mushrooms: &[Option<MushroomCell>],
    cell: IVec2,
) -> Option<MushroomCell> {
    if cell.x < 0 || cell.y < 0 || cell.x >= config.grid_w || cell.y >= config.grid_h {
        return None;
    }
    mushrooms[(cell.y * config.grid_w + cell.x) as usize]
}

fn pos_to_cell_in_bounds(config: &GameConfig, pos: Vec2) -> Option<IVec2> {
    let cell = ivec2(pos.x.round() as i32, pos.y.round() as i32);
    if cell.x < 0 || cell.y < 0 || cell.x >= config.grid_w || cell.y >= config.grid_h {
        return None;
    }
    Some(cell)
}

fn spider_score_for_range(config: &GameConfig, player_pos: Vec2, spider_pos: Vec2) -> u32 {
    let distance = player_pos.distance(spider_pos);
    if distance <= 4.0 {
        config.score_spider_near
    } else if distance <= 8.0 {
        config.score_spider_mid
    } else {
        config.score_spider_far
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> GameConfig {
        GameConfig {
            fixed_dt: 1.0 / 60.0,
            ..GameConfig::default()
        }
    }

    #[test]
    fn centipede_split_promotes_new_heads_and_spawns_mushroom() {
        let config = test_config();
        let mut world = World::new(config, 1);
        world.centipede_chains.clear();
        world.mushrooms.iter_mut().for_each(|slot| *slot = None);

        world.centipede_chains.push(CentipedeChain {
            segments: vec![
                Segment {
                    pos: ivec2(3, 6),
                    is_head: true,
                },
                Segment {
                    pos: ivec2(4, 6),
                    is_head: false,
                },
                Segment {
                    pos: ivec2(5, 6),
                    is_head: false,
                },
                Segment {
                    pos: ivec2(6, 6),
                    is_head: false,
                },
            ],
            dir: 1,
            mode: CentipedeMode::Normal,
            entered_player_area: false,
            step_interval: world.config.centipede_step_interval,
            step_timer: 0.0,
            origin_main: true,
        });

        world.apply_chain_hit(0, 2);
        assert_eq!(world.centipede_chains.len(), 2);
        assert!(world.centipede_chains[0].segments[0].is_head);
        assert!(world.centipede_chains[1].segments[0].is_head);
        assert_eq!(world.mushroom_cell(ivec2(5, 6)).unwrap().hp, 4);
    }

    #[test]
    fn mushroom_damage_and_death_regeneration_awards_points() {
        let mut world = World::new(test_config(), 2);
        world.mushrooms.iter_mut().for_each(|slot| *slot = None);
        world.score = 0;

        let cell = ivec2(4, world.config.player_area_start_row());
        world.spawn_mushroom(cell, true, 2);
        assert!(world.projectile_hits_mushroom(cell));
        assert_eq!(world.mushroom_cell(cell).unwrap().hp, 1);
        assert_eq!(world.score, 1);

        world.handle_player_death();
        let regenerated = world.mushroom_cell(cell).unwrap();
        assert!(!regenerated.poisoned);
        assert_eq!(regenerated.hp, 4);
        assert!(world.score >= 6);
    }

    #[test]
    fn round_progression_cycles_full_to_heads_then_resets() {
        let mut director = RoundDirector::new(12);
        let mut observed = Vec::new();
        for _ in 0..13 {
            observed.push((
                director.current_main_length(),
                director.current_bonus_heads(),
            ));
            director.advance();
        }
        assert_eq!(observed[0], (12, 0));
        assert_eq!(observed[1], (11, 1));
        assert_eq!(observed[10], (2, 10));
        assert_eq!(observed[11], (1, 11));
        assert_eq!(observed[12], (12, 0));
    }

    #[test]
    fn poison_mushroom_forces_dive_then_returns_normal_at_bottom() {
        let mut world = World::new(test_config(), 3);
        world.centipede_chains.clear();
        world.mushrooms.iter_mut().for_each(|slot| *slot = None);

        world.centipede_chains.push(CentipedeChain {
            segments: vec![
                Segment {
                    pos: ivec2(5, 4),
                    is_head: true,
                },
                Segment {
                    pos: ivec2(4, 4),
                    is_head: false,
                },
            ],
            dir: 1,
            mode: CentipedeMode::Normal,
            entered_player_area: false,
            step_interval: world.config.centipede_step_interval,
            step_timer: 0.0,
            origin_main: true,
        });
        world.spawn_mushroom(ivec2(6, 4), true, 4);

        world.step_chain_once(0);
        assert_eq!(world.centipede_chains[0].mode, CentipedeMode::PoisonDive);
        for _ in 0..(world.config.grid_h + 4) {
            world.step_chain_once(0);
        }
        assert_eq!(world.centipede_chains[0].mode, CentipedeMode::Normal);
        assert!(
            world.centipede_chains[0].segments[0].pos.y >= world.config.player_area_start_row()
        );
    }

    #[test]
    fn scoring_table_matches_configured_values() {
        let config = test_config();
        let mut world = World::new(config.clone(), 4);
        world.score = 0;
        world.centipede_chains.clear();
        world.centipede_chains.push(CentipedeChain {
            segments: vec![
                Segment {
                    pos: ivec2(5, 5),
                    is_head: true,
                },
                Segment {
                    pos: ivec2(4, 5),
                    is_head: false,
                },
            ],
            dir: 1,
            mode: CentipedeMode::Normal,
            entered_player_area: false,
            step_interval: world.config.centipede_step_interval,
            step_timer: 0.0,
            origin_main: true,
        });

        world.apply_chain_hit(0, 1);
        world.apply_chain_hit(0, 0);
        assert_eq!(world.score, config.score_body + config.score_head);

        let near = spider_score_for_range(&config, vec2(10.0, 26.0), vec2(11.0, 26.0));
        let mid = spider_score_for_range(&config, vec2(10.0, 26.0), vec2(16.0, 26.0));
        let far = spider_score_for_range(&config, vec2(10.0, 26.0), vec2(25.0, 26.0));
        assert_eq!(near, config.score_spider_near);
        assert_eq!(mid, config.score_spider_mid);
        assert_eq!(far, config.score_spider_far);
        assert_eq!(config.score_scorpion, 1000);
        assert_eq!(config.score_flea, 200);
    }

    #[test]
    fn spawn_guards_handle_flea_and_side_head_conditions() {
        let mut world = World::new(test_config(), 5);
        world.enemies.clear();
        world.mushrooms.iter_mut().for_each(|slot| *slot = None);
        world.flea_spawn_timer = 0.0;
        world.spider_spawn_timer = 999.0;
        world.scorpion_spawn_timer = 999.0;
        world.side_head_spawn_timer = 0.0;
        world.bottom_phase_active = true;
        world.centipede_chains.clear();
        world.centipede_chains.push(CentipedeChain {
            segments: vec![Segment {
                pos: ivec2(2, world.config.player_area_start_row()),
                is_head: true,
            }],
            dir: 1,
            mode: CentipedeMode::Normal,
            entered_player_area: true,
            step_interval: world.config.centipede_step_interval,
            step_timer: 0.0,
            origin_main: true,
        });

        world.handle_spawns(0.1);
        let flea_count = world
            .enemies
            .iter()
            .filter(|enemy| matches!(enemy, Enemy::Flea(_)))
            .count();
        let head_count = world
            .enemies
            .iter()
            .filter(|enemy| matches!(enemy, Enemy::DetachedHead(_)))
            .count();
        assert_eq!(flea_count, 1);
        assert_eq!(head_count, 1);

        world.flea_spawn_timer = 0.0;
        world.side_head_spawn_timer = 0.0;
        world.centipede_chains.clear();
        let existing_heads = world
            .enemies
            .iter()
            .filter(|enemy| matches!(enemy, Enemy::DetachedHead(_)))
            .count();
        world.handle_spawns(0.1);
        let heads_after = world
            .enemies
            .iter()
            .filter(|enemy| matches!(enemy, Enemy::DetachedHead(_)))
            .count();
        assert_eq!(heads_after, existing_heads);
    }
}
