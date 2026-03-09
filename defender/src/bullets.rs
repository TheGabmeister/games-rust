use crate::constants::*;
use crate::world::Camera;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum BulletOwner {
    Player,
    Enemy,
}

pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub owner: BulletOwner,
    pub alive: bool,
    pub lifetime: f32,
}

impl Bullet {
    pub fn new_player(pos: Vec2, vel: Vec2) -> Self {
        Bullet {
            pos,
            vel,
            owner: BulletOwner::Player,
            alive: true,
            lifetime: PLAYER_BULLET_LIFETIME,
        }
    }

    pub fn new_enemy_bomb(pos: Vec2) -> Self {
        Bullet {
            pos,
            vel: Vec2::new(0.0, ENEMY_BOMB_SPEED),
            owner: BulletOwner::Enemy,
            alive: true,
            lifetime: 4.0,
        }
    }

    pub fn new_enemy_bullet(pos: Vec2, vel: Vec2) -> Self {
        Bullet {
            pos,
            vel,
            owner: BulletOwner::Enemy,
            alive: true,
            lifetime: 2.5,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos.x = (self.pos.x + self.vel.x * dt).rem_euclid(WORLD_WIDTH);
        self.pos.y += self.vel.y * dt;
        self.lifetime -= dt;
        if self.lifetime <= 0.0 {
            self.alive = false;
        }
    }

    pub fn aabb(&self) -> Rect {
        Rect::new(
            self.pos.x - BULLET_HALF_W,
            self.pos.y - BULLET_HALF_H,
            BULLET_HALF_W * 2.0,
            BULLET_HALF_H * 2.0,
        )
    }

    pub fn draw(&self, camera: &Camera) {
        if !self.alive {
            return;
        }
        let sx = camera.world_to_screen_x(self.pos.x);
        let sy = camera.world_to_screen_y(self.pos.y);

        match self.owner {
            BulletOwner::Player => {
                draw_rectangle(
                    sx - BULLET_HALF_W,
                    sy - BULLET_HALF_H,
                    BULLET_HALF_W * 2.0,
                    BULLET_HALF_H * 2.0,
                    WHITE,
                );
            }
            BulletOwner::Enemy => {
                draw_circle(sx, sy, 4.0, Color::new(1.0, 0.4, 0.0, 1.0));
            }
        }
    }
}
