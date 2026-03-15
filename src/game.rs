use rand::Rng;
use raylib::prelude::*;

use crate::audio::{AudioFx, BackgroundMusic};

pub const SCREEN_WIDTH: i32 = 960;
pub const SCREEN_HEIGHT: i32 = 540;
const GROUND_HEIGHT: i32 = 110;
const PLAYER_X: f32 = 140.0;
const PLAYER_RADIUS: f32 = 24.0;
const PLAYER_BASE_Y: f32 = SCREEN_HEIGHT as f32 - GROUND_HEIGHT as f32 - PLAYER_RADIUS;
const GRAVITY: f32 = 0.60;
const JUMP_VELOCITY: f32 = -13.5;
const DOUBLE_JUMP_VELOCITY: f32 = -12.2;
const START_SPEED: f32 = 6.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    Title,
    Playing,
    GameOver,
}

#[derive(Clone, Copy)]
enum ObstacleKind {
    Crate,
    Spike,
    Pillar,
    Drone,
}

struct Player {
    y: f32,
    velocity: f32,
    air_jump_ready: bool,
    stretch_timer: i32,
}

struct Obstacle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    speed: f32,
    kind: ObstacleKind,
    passed: bool,
}

struct Coin {
    x: f32,
    y: f32,
    radius: f32,
    speed: f32,
    bob_phase: f32,
}

struct Cloud {
    x: f32,
    y: f32,
    radius: f32,
    speed: f32,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: i32,
    radius: f32,
    color: Color,
}

pub struct Game {
    scene: Scene,
    player: Player,
    obstacles: Vec<Obstacle>,
    coins: Vec<Coin>,
    clouds: Vec<Cloud>,
    particles: Vec<Particle>,
    score_distance: f32,
    bonus_score: i32,
    best_score: i32,
    cleared_obstacles: i32,
    spawn_timer: i32,
    next_spawn_in: i32,
    coin_timer: i32,
    next_coin_in: i32,
    frame_count: i32,
    world_speed: f32,
}

impl Player {
    fn new() -> Self {
        Self {
            y: PLAYER_BASE_Y,
            velocity: 0.0,
            air_jump_ready: true,
            stretch_timer: 0,
        }
    }

    fn on_ground(&self) -> bool {
        self.y >= PLAYER_BASE_Y - 0.1
    }
}

impl Game {
    pub fn new(rng: &mut impl Rng) -> Self {
        let mut game = Self {
            scene: Scene::Title,
            player: Player::new(),
            obstacles: Vec::new(),
            coins: Vec::new(),
            clouds: Vec::new(),
            particles: Vec::new(),
            score_distance: 0.0,
            bonus_score: 0,
            best_score: 0,
            cleared_obstacles: 0,
            spawn_timer: 0,
            next_spawn_in: 90,
            coin_timer: 0,
            next_coin_in: 180,
            frame_count: 0,
            world_speed: START_SPEED,
        };

        game.populate_clouds(rng);
        game
    }

    pub fn scene(&self) -> Scene {
        self.scene
    }

    pub fn reset_run(&mut self, rng: &mut impl Rng) {
        self.scene = Scene::Playing;
        self.player = Player::new();
        self.obstacles.clear();
        self.coins.clear();
        self.particles.clear();
        self.score_distance = 0.0;
        self.bonus_score = 0;
        self.cleared_obstacles = 0;
        self.spawn_timer = 0;
        self.next_spawn_in = rng.random_range(70..110);
        self.coin_timer = 0;
        self.next_coin_in = rng.random_range(120..220);
        self.frame_count = 0;
        self.world_speed = START_SPEED;
    }

    fn total_score(&self) -> i32 {
        self.score_distance as i32 + self.bonus_score + self.cleared_obstacles * 10
    }

    fn populate_clouds(&mut self, rng: &mut impl Rng) {
        self.clouds.clear();
        for i in 0..6 {
            self.clouds.push(Cloud {
                x: i as f32 * 190.0 + rng.random_range(0.0..60.0),
                y: rng.random_range(50.0..200.0),
                radius: rng.random_range(24.0..52.0),
                speed: rng.random_range(0.4..1.4),
            });
        }
    }

    pub fn update_title(&mut self, rng: &mut impl Rng) {
        self.update_clouds(rng);
        self.update_particles();
    }

    pub fn update_playing(&mut self, rl: &RaylibHandle, rng: &mut impl Rng, audio: &AudioFx) {
        self.frame_count += 1;
        self.world_speed = (START_SPEED + self.frame_count as f32 * 0.006).min(13.0);
        self.score_distance += self.world_speed * 0.18;

        self.update_clouds(rng);
        self.handle_input(rl, audio);
        self.update_player(audio);
        self.update_spawns(rng);
        self.update_obstacles(audio);
        self.update_coins();
        self.update_particles();

        if self.check_collisions(audio) {
            self.scene = Scene::GameOver;
            self.best_score = self.best_score.max(self.total_score());
            self.spawn_game_over_burst();
            audio.play_hit();
        }
    }

    pub fn update_game_over(&mut self, rng: &mut impl Rng) {
        self.update_clouds(rng);
        self.update_particles();
    }

    fn handle_input(&mut self, rl: &RaylibHandle, audio: &AudioFx) {
        let jump_pressed = rl.is_key_pressed(KeyboardKey::KEY_SPACE)
            || rl.is_key_pressed(KeyboardKey::KEY_UP)
            || rl.is_key_pressed(KeyboardKey::KEY_W);

        if !jump_pressed {
            return;
        }

        if self.player.on_ground() {
            self.player.velocity = JUMP_VELOCITY;
            self.player.stretch_timer = 10;
            self.spawn_jump_particles(Color::new(218, 181, 120, 255), 6);
            audio.play_jump();
        } else if self.player.air_jump_ready {
            self.player.velocity = DOUBLE_JUMP_VELOCITY;
            self.player.air_jump_ready = false;
            self.player.stretch_timer = 12;
            self.spawn_jump_particles(Color::new(255, 220, 110, 255), 10);
            audio.play_double_jump();
        }
    }

    fn update_player(&mut self, audio: &AudioFx) {
        let was_on_ground = self.player.on_ground();
        let previous_velocity = self.player.velocity;

        self.player.velocity += GRAVITY;
        self.player.y += self.player.velocity;

        if self.player.y > PLAYER_BASE_Y {
            self.player.y = PLAYER_BASE_Y;
            self.player.velocity = 0.0;

            if !was_on_ground && previous_velocity > 2.0 {
                self.player.air_jump_ready = true;
                self.player.stretch_timer = 8;
                self.spawn_jump_particles(Color::new(185, 150, 100, 255), 8);
                audio.play_land();
            }
        }

        if self.player.stretch_timer > 0 {
            self.player.stretch_timer -= 1;
        }
    }

    fn update_spawns(&mut self, rng: &mut impl Rng) {
        self.spawn_timer += 1;
        self.coin_timer += 1;

        if self.spawn_timer >= self.next_spawn_in {
            self.spawn_obstacle(rng);
            self.spawn_timer = 0;

            let min_gap = (58.0 - self.frame_count as f32 * 0.01).max(34.0) as i32;
            let max_gap = (112.0 - self.frame_count as f32 * 0.015).max((min_gap + 10) as f32) as i32;
            self.next_spawn_in = rng.random_range(min_gap..=max_gap);
        }

        if self.coin_timer >= self.next_coin_in {
            self.spawn_coin(rng);
            self.coin_timer = 0;
            self.next_coin_in = rng.random_range(130..260);
        }
    }

    fn spawn_obstacle(&mut self, rng: &mut impl Rng) {
        let roll = rng.random_range(0..100);
        let kind = match roll {
            0..=32 => ObstacleKind::Crate,
            33..=57 => ObstacleKind::Spike,
            58..=82 => ObstacleKind::Pillar,
            _ => ObstacleKind::Drone,
        };

        let (width, height, y) = match kind {
            ObstacleKind::Crate => (50.0, 52.0, SCREEN_HEIGHT as f32 - GROUND_HEIGHT as f32 - 52.0),
            ObstacleKind::Spike => (44.0, 38.0, SCREEN_HEIGHT as f32 - GROUND_HEIGHT as f32 - 38.0),
            ObstacleKind::Pillar => (34.0, 96.0, SCREEN_HEIGHT as f32 - GROUND_HEIGHT as f32 - 96.0),
            ObstacleKind::Drone => {
                let altitude = rng.random_range(95.0..170.0);
                (54.0, 26.0, PLAYER_BASE_Y - altitude)
            }
        };

        self.obstacles.push(Obstacle {
            x: SCREEN_WIDTH as f32 + rng.random_range(0.0..120.0),
            y,
            width,
            height,
            speed: self.world_speed + rng.random_range(0.0..2.2),
            kind,
            passed: false,
        });
    }

    fn spawn_coin(&mut self, rng: &mut impl Rng) {
        let y = match rng.random_range(0..3) {
            0 => PLAYER_BASE_Y - rng.random_range(55.0..95.0),
            1 => PLAYER_BASE_Y - rng.random_range(110.0..155.0),
            _ => PLAYER_BASE_Y - rng.random_range(160.0..210.0),
        };

        self.coins.push(Coin {
            x: SCREEN_WIDTH as f32 + rng.random_range(40.0..140.0),
            y,
            radius: 13.0,
            speed: self.world_speed + rng.random_range(0.4..1.8),
            bob_phase: rng.random_range(0.0..std::f32::consts::TAU),
        });
    }

    fn update_obstacles(&mut self, audio: &AudioFx) {
        let mut cleared_positions = Vec::new();

        for obstacle in &mut self.obstacles {
            obstacle.x -= obstacle.speed;

            if !obstacle.passed && obstacle.x + obstacle.width < PLAYER_X - PLAYER_RADIUS {
                obstacle.passed = true;
                self.cleared_obstacles += 1;
                cleared_positions.push((obstacle.x + obstacle.width * 0.5, obstacle.y + 12.0));
            }
        }

        let played_clear = !cleared_positions.is_empty();

        for (x, y) in cleared_positions {
            self.spawn_clear_particles(x, y);
        }

        if played_clear {
            audio.play_clear();
        }

        self.obstacles.retain(|obstacle| obstacle.x + obstacle.width > -60.0);
    }

    fn update_coins(&mut self) {
        for coin in &mut self.coins {
            coin.x -= coin.speed;
            coin.bob_phase += 0.08;
        }

        self.coins.retain(|coin| coin.x + coin.radius > -30.0);
    }

    fn update_clouds(&mut self, rng: &mut impl Rng) {
        for cloud in &mut self.clouds {
            cloud.x -= cloud.speed + self.world_speed * 0.08;

            if cloud.x + cloud.radius * 3.0 < 0.0 {
                cloud.x = SCREEN_WIDTH as f32 + rng.random_range(50.0..220.0);
                cloud.y = rng.random_range(50.0..200.0);
                cloud.radius = rng.random_range(24.0..52.0);
                cloud.speed = rng.random_range(0.4..1.4);
            }
        }
    }

    fn update_particles(&mut self) {
        for particle in &mut self.particles {
            particle.x += particle.vx;
            particle.y += particle.vy;
            particle.vy += 0.16;
            particle.life -= 1;
            particle.radius *= 0.98;
        }

        self.particles.retain(|particle| particle.life > 0 && particle.radius > 0.8);
    }

    fn check_collisions(&mut self, audio: &AudioFx) -> bool {
        for obstacle in &self.obstacles {
            let hit = match obstacle.kind {
                ObstacleKind::Spike => circle_triangle_collision(
                    PLAYER_X,
                    self.player.y,
                    PLAYER_RADIUS - 3.0,
                    Vector2::new(obstacle.x, obstacle.y + obstacle.height),
                    Vector2::new(obstacle.x + obstacle.width * 0.5, obstacle.y),
                    Vector2::new(obstacle.x + obstacle.width, obstacle.y + obstacle.height),
                ),
                _ => circle_rect_collision(
                    PLAYER_X,
                    self.player.y,
                    PLAYER_RADIUS - 2.0,
                    obstacle.x,
                    obstacle.y,
                    obstacle.width,
                    obstacle.height,
                ),
            };

            if hit {
                return true;
            }
        }

        let mut collected = Vec::new();
        for (index, coin) in self.coins.iter().enumerate() {
            let coin_y = coin.y + coin.bob_phase.sin() * 6.0;
            let dx = PLAYER_X - coin.x;
            let dy = self.player.y - coin_y;
            let total_radius = PLAYER_RADIUS + coin.radius;

            if dx * dx + dy * dy <= total_radius * total_radius {
                collected.push(index);
            }
        }

        let played_coin = !collected.is_empty();

        for index in collected.into_iter().rev() {
            let coin = self.coins.remove(index);
            self.bonus_score += 25;
            self.spawn_coin_particles(coin.x, coin.y);
        }

        if played_coin {
            audio.play_coin();
        }

        false
    }

    fn spawn_jump_particles(&mut self, color: Color, amount: usize) {
        let mut rng = rand::rng();
        for _ in 0..amount {
            self.particles.push(Particle {
                x: PLAYER_X + rng.random_range(-8.0..8.0),
                y: PLAYER_BASE_Y + PLAYER_RADIUS - 2.0,
                vx: rng.random_range(-2.8..2.8),
                vy: rng.random_range(-3.5..-1.0),
                life: rng.random_range(18..34),
                radius: rng.random_range(2.4..4.8),
                color,
            });
        }
    }

    fn spawn_clear_particles(&mut self, x: f32, y: f32) {
        let mut rng = rand::rng();
        for _ in 0..5 {
            self.particles.push(Particle {
                x,
                y,
                vx: rng.random_range(-1.5..2.0),
                vy: rng.random_range(-2.6..-0.6),
                life: rng.random_range(16..28),
                radius: rng.random_range(2.0..3.8),
                color: Color::new(220, 244, 255, 255),
            });
        }
    }

    fn spawn_coin_particles(&mut self, x: f32, y: f32) {
        let mut rng = rand::rng();
        for _ in 0..10 {
            self.particles.push(Particle {
                x,
                y,
                vx: rng.random_range(-3.2..3.2),
                vy: rng.random_range(-3.5..1.5),
                life: rng.random_range(18..36),
                radius: rng.random_range(2.0..4.8),
                color: Color::new(255, 210, 70, 255),
            });
        }
    }

    fn spawn_game_over_burst(&mut self) {
        let mut rng = rand::rng();
        for _ in 0..20 {
            self.particles.push(Particle {
                x: PLAYER_X,
                y: self.player.y,
                vx: rng.random_range(-4.5..4.5),
                vy: rng.random_range(-4.0..2.0),
                life: rng.random_range(18..42),
                radius: rng.random_range(2.0..6.0),
                color: Color::new(255, 110, 110, 255),
            });
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, audio: &AudioFx, music: &BackgroundMusic) {
        d.clear_background(Color::new(160, 220, 255, 255));
        self.draw_backdrop(d);
        self.draw_ground(d);
        self.draw_coins(d);
        self.draw_obstacles(d);
        self.draw_particles(d);
        self.draw_player(d);
        self.draw_hud(d, audio, music);

        match self.scene {
            Scene::Title => self.draw_title_overlay(d),
            Scene::GameOver => self.draw_game_over_overlay(d),
            Scene::Playing => {}
        }
    }

    fn draw_backdrop(&self, d: &mut RaylibDrawHandle) {
        d.draw_circle(780, 96, 54.0, Color::new(255, 222, 120, 255));
        d.draw_circle(780, 96, 40.0, Color::new(255, 238, 180, 255));

        d.draw_circle(120, SCREEN_HEIGHT - 50, 220.0, Color::new(122, 198, 143, 255));
        d.draw_circle(390, SCREEN_HEIGHT - 36, 180.0, Color::new(110, 190, 138, 255));
        d.draw_circle(690, SCREEN_HEIGHT - 42, 210.0, Color::new(95, 176, 128, 255));
        d.draw_circle(910, SCREEN_HEIGHT - 28, 168.0, Color::new(110, 190, 138, 255));

        for cloud in &self.clouds {
            let x = cloud.x as i32;
            let y = cloud.y as i32;
            let r = cloud.radius;
            d.draw_circle(x, y, r, Color::new(255, 255, 255, 230));
            d.draw_circle((cloud.x + r * 0.9) as i32, (cloud.y + 5.0) as i32, r * 0.82, Color::new(255, 255, 255, 220));
            d.draw_circle((cloud.x - r * 0.9) as i32, (cloud.y + 10.0) as i32, r * 0.72, Color::new(248, 251, 255, 220));
        }
    }

    fn draw_ground(&self, d: &mut RaylibDrawHandle) {
        let ground_y = SCREEN_HEIGHT - GROUND_HEIGHT;
        d.draw_rectangle(0, ground_y, SCREEN_WIDTH, GROUND_HEIGHT, Color::new(111, 88, 62, 255));
        d.draw_rectangle(0, ground_y + 62, SCREEN_WIDTH, GROUND_HEIGHT - 62, Color::new(96, 74, 53, 255));
        d.draw_rectangle(0, ground_y, SCREEN_WIDTH, 18, Color::new(110, 152, 78, 255));
        d.draw_rectangle(0, ground_y + 18, SCREEN_WIDTH, 10, Color::new(84, 110, 60, 255));
        d.draw_rectangle(0, ground_y + 28, SCREEN_WIDTH, 8, Color::new(98, 81, 59, 255));

        let turf_offset = -((self.frame_count as f32 * self.world_speed * 0.42) as i32 % 52);
        let grass_offset = -((self.frame_count as f32 * self.world_speed * 0.95) as i32 % 38);
        let pebble_offset = -((self.frame_count as f32 * self.world_speed * 0.70) as i32 % 64);
        let stratum_offset = -((self.frame_count as f32 * self.world_speed * 0.28) as i32 % 120);

        let mut mound_x = turf_offset - 52;
        while mound_x < SCREEN_WIDTH + 72 {
            d.draw_ellipse(mound_x + 16, ground_y + 9, 24.0, 9.0, Color::new(98, 144, 71, 255));
            d.draw_ellipse(mound_x + 38, ground_y + 8, 20.0, 8.0, Color::new(107, 154, 76, 255));
            d.draw_ellipse(mound_x + 58, ground_y + 10, 22.0, 9.0, Color::new(93, 136, 67, 255));
            mound_x += 52;
        }

        let mut blade_x = grass_offset;
        while blade_x < SCREEN_WIDTH + 32 {
            d.draw_line(blade_x + 4, ground_y + 18, blade_x + 3, ground_y + 9, Color::new(66, 112, 52, 220));
            d.draw_line(blade_x + 8, ground_y + 18, blade_x + 8, ground_y + 7, Color::new(74, 123, 56, 235));
            d.draw_line(blade_x + 12, ground_y + 18, blade_x + 14, ground_y + 10, Color::new(70, 116, 54, 220));
            blade_x += 32;
        }

        let mut stratum_x = stratum_offset - 80;
        while stratum_x < SCREEN_WIDTH + 120 {
            d.draw_ellipse(stratum_x + 36, ground_y + 46, 42.0, 7.0, Color::new(141, 109, 77, 255));
            d.draw_ellipse(stratum_x + 72, ground_y + 80, 38.0, 6.0, Color::new(132, 101, 71, 255));
            d.draw_ellipse(stratum_x + 18, ground_y + 98, 28.0, 5.0, Color::new(87, 68, 48, 170));
            stratum_x += 120;
        }

        let mut pebble_x = pebble_offset;
        while pebble_x < SCREEN_WIDTH + 64 {
            d.draw_ellipse(pebble_x + 12, ground_y + 54, 7.0, 4.0, Color::new(149, 124, 98, 255));
            d.draw_ellipse(pebble_x + 34, ground_y + 73, 5.5, 3.5, Color::new(128, 103, 82, 255));
            d.draw_ellipse(pebble_x + 22, ground_y + 96, 8.0, 4.5, Color::new(118, 94, 73, 255));
            d.draw_circle(pebble_x + 44, ground_y + 107, 2.5, Color::new(140, 112, 88, 255));
            pebble_x += 64;
        }

        let mut root_x = stratum_offset - 20;
        while root_x < SCREEN_WIDTH + 90 {
            d.draw_line(root_x + 20, ground_y + 32, root_x + 8, ground_y + 52, Color::new(105, 78, 57, 160));
            d.draw_line(root_x + 20, ground_y + 32, root_x + 31, ground_y + 60, Color::new(99, 74, 54, 145));
            root_x += 90;
        }

        d.draw_line(0, ground_y, SCREEN_WIDTH, ground_y, Color::new(59, 97, 48, 255));
        d.draw_line(0, ground_y + 18, SCREEN_WIDTH, ground_y + 18, Color::new(82, 102, 60, 255));
        d.draw_line(0, ground_y + 28, SCREEN_WIDTH, ground_y + 28, Color::new(89, 68, 48, 255));
    }

    fn draw_player(&self, d: &mut RaylibDrawHandle) {
        let y = self.player.y;
        let shadow_scale = if self.player.on_ground() { 1.0 } else { 0.72 };
        d.draw_circle(PLAYER_X as i32, (PLAYER_BASE_Y + PLAYER_RADIUS + 8.0) as i32, 18.0 * shadow_scale, Color::new(0, 0, 0, 55));

        d.draw_circle(PLAYER_X as i32, y as i32, PLAYER_RADIUS + 3.0, Color::new(28, 30, 56, 255));
        d.draw_circle(PLAYER_X as i32, y as i32, PLAYER_RADIUS, Color::new(255, 108, 108, 255));
        d.draw_circle((PLAYER_X - 7.0) as i32, (y - 6.0) as i32, 6.2, Color::WHITE);
        d.draw_circle((PLAYER_X + 6.0) as i32, (y - 7.0) as i32, 6.2, Color::WHITE);
        d.draw_circle((PLAYER_X - 6.0) as i32, (y - 5.0) as i32, 2.4, Color::BLACK);
        d.draw_circle((PLAYER_X + 7.0) as i32, (y - 6.0) as i32, 2.4, Color::BLACK);

        if self.player.air_jump_ready && !self.player.on_ground() {
            d.draw_circle((PLAYER_X + 22.0) as i32, (y - 24.0) as i32, 6.0, Color::new(255, 235, 120, 255));
            d.draw_circle((PLAYER_X + 22.0) as i32, (y - 24.0) as i32, 2.5, Color::new(255, 255, 245, 255));
        }
    }

    fn draw_obstacles(&self, d: &mut RaylibDrawHandle) {
        for obstacle in &self.obstacles {
            match obstacle.kind {
                ObstacleKind::Crate => {
                    d.draw_rectangle(obstacle.x as i32, obstacle.y as i32, obstacle.width as i32, obstacle.height as i32, Color::new(78, 54, 34, 255));
                    d.draw_rectangle((obstacle.x + 4.0) as i32, (obstacle.y + 4.0) as i32, obstacle.width as i32 - 8, obstacle.height as i32 - 8, Color::new(145, 104, 62, 255));
                    d.draw_line((obstacle.x + 8.0) as i32, (obstacle.y + 8.0) as i32, (obstacle.x + obstacle.width - 8.0) as i32, (obstacle.y + obstacle.height - 8.0) as i32, Color::new(92, 67, 44, 255));
                    d.draw_line((obstacle.x + obstacle.width - 8.0) as i32, (obstacle.y + 8.0) as i32, (obstacle.x + 8.0) as i32, (obstacle.y + obstacle.height - 8.0) as i32, Color::new(92, 67, 44, 255));
                }
                ObstacleKind::Spike => {
                    let p1 = Vector2::new(obstacle.x, obstacle.y + obstacle.height);
                    let p2 = Vector2::new(obstacle.x + obstacle.width * 0.5, obstacle.y);
                    let p3 = Vector2::new(obstacle.x + obstacle.width, obstacle.y + obstacle.height);
                    d.draw_triangle(p2, p1, p3, Color::new(40, 42, 58, 255));
                    d.draw_triangle(
                        Vector2::new(p2.x, p2.y + 6.0),
                        Vector2::new(p1.x + 6.0, p1.y - 4.0),
                        Vector2::new(p3.x - 6.0, p3.y - 4.0),
                        Color::new(220, 86, 92, 255),
                    );
                }
                ObstacleKind::Pillar => {
                    d.draw_rectangle(obstacle.x as i32, obstacle.y as i32, obstacle.width as i32, obstacle.height as i32, Color::new(50, 63, 92, 255));
                    d.draw_rectangle((obstacle.x + 5.0) as i32, (obstacle.y + 8.0) as i32, obstacle.width as i32 - 10, obstacle.height as i32 - 14, Color::new(89, 108, 150, 255));
                    d.draw_rectangle((obstacle.x - 5.0) as i32, (obstacle.y - 8.0) as i32, obstacle.width as i32 + 10, 10, Color::new(50, 63, 92, 255));
                }
                ObstacleKind::Drone => {
                    d.draw_rectangle(obstacle.x as i32, obstacle.y as i32, obstacle.width as i32, obstacle.height as i32, Color::new(45, 46, 74, 255));
                    d.draw_rectangle((obstacle.x + 5.0) as i32, (obstacle.y + 5.0) as i32, obstacle.width as i32 - 10, obstacle.height as i32 - 10, Color::new(115, 231, 244, 255));
                    d.draw_line((obstacle.x - 4.0) as i32, (obstacle.y + obstacle.height * 0.5) as i32, (obstacle.x + obstacle.width + 4.0) as i32, (obstacle.y + obstacle.height * 0.5) as i32, Color::new(240, 248, 255, 255));
                }
            }
        }
    }

    fn draw_coins(&self, d: &mut RaylibDrawHandle) {
        for coin in &self.coins {
            let bob_y = coin.y + coin.bob_phase.sin() * 6.0;
            let spin = coin.bob_phase.cos().abs();
            let face_width = (coin.radius * (0.35 + spin * 0.65)).max(4.0);
            let face_height = coin.radius + 1.0;
            let x = coin.x as i32;
            let y = bob_y as i32;

            d.draw_ellipse(x, y + 2, face_width + 2.0, face_height + 2.0, Color::new(120, 78, 10, 80));
            d.draw_ellipse(x + 2, y + 1, face_width, face_height, Color::new(205, 140, 28, 255));
            d.draw_ellipse(x, y, face_width, face_height, Color::new(255, 208, 58, 255));
            d.draw_ellipse(x, y, (face_width - 2.5).max(2.0), (face_height - 2.5).max(2.0), Color::new(255, 232, 120, 255));
            d.draw_ellipse_lines(x, y, face_width, face_height, Color::new(166, 110, 18, 255));
            d.draw_ellipse_lines(
                x,
                y,
                (face_width - 3.5).max(1.0),
                (face_height - 3.5).max(1.0),
                Color::new(240, 190, 46, 255),
            );

            for ridge in -3..=3 {
                let ridge_x = x + (ridge as f32 * face_width / 4.2) as i32;
                let ridge_height = (face_height * 0.15) as i32;
                d.draw_line(
                    ridge_x,
                    y - face_height as i32 + 3,
                    ridge_x,
                    y - face_height as i32 + 3 + ridge_height,
                    Color::new(176, 118, 24, 220),
                );
                d.draw_line(
                    ridge_x,
                    y + face_height as i32 - 3,
                    ridge_x,
                    y + face_height as i32 - 3 - ridge_height,
                    Color::new(176, 118, 24, 220),
                );
            }

            if face_width > coin.radius * 0.55 {
                d.draw_circle(x - (face_width * 0.32) as i32, y - (face_height * 0.34) as i32, 3.6, Color::new(255, 249, 214, 220));
                d.draw_circle(x - (face_width * 0.12) as i32, y - (face_height * 0.18) as i32, 2.1, Color::new(255, 245, 200, 180));
                d.draw_text("$", x - 6, y - 12, 24, Color::new(160, 106, 8, 255));
            } else {
                d.draw_rectangle(x - 1, y - face_height as i32 + 3, 2, face_height as i32 * 2 - 6, Color::new(255, 239, 182, 190));
            }
        }
    }

    fn draw_particles(&self, d: &mut RaylibDrawHandle) {
        for particle in &self.particles {
            d.draw_circle(particle.x as i32, particle.y as i32, particle.radius, particle.color);
        }
    }

    fn draw_hud(&self, d: &mut RaylibDrawHandle, audio: &AudioFx, music: &BackgroundMusic) {
        d.draw_rectangle(18, 16, 235, 84, Color::new(255, 255, 255, 165));
        d.draw_text(&format!("Score  {}", self.total_score()), 30, 26, 26, Color::new(30, 44, 65, 255));
        d.draw_text(&format!("Best   {}", self.best_score.max(self.total_score())), 30, 56, 20, Color::new(30, 44, 65, 255));
        d.draw_text(&format!("Speed  {:.1}", self.world_speed), 30, 78, 18, Color::new(30, 44, 65, 255));

        let jump_hint = if self.player.air_jump_ready { "Double jump ready" } else { "Double jump used" };
        d.draw_text(jump_hint, SCREEN_WIDTH - 220, 24, 20, Color::new(40, 54, 74, 255));
        d.draw_text(if audio.is_muted() { "M = sound off" } else { "M = sound on" }, SCREEN_WIDTH - 220, 50, 20, Color::new(40, 54, 74, 255));
        d.draw_text(if music.is_muted() { "B = music off" } else { "B = music on" }, SCREEN_WIDTH - 220, 76, 20, Color::new(40, 54, 74, 255));
    }

    fn draw_title_overlay(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle(170, 95, 620, 400, Color::new(255, 255, 255, 185));
        d.draw_text("Jumper", 275, 126, 42, Color::new(36, 46, 70, 255));
        d.draw_text("New goodies:", 300, 186, 28, Color::new(55, 74, 96, 255));
        d.draw_text("- Multiple obstacle types", 300, 224, 24, Color::new(55, 74, 96, 255));
        d.draw_text("- Collectible coins", 300, 258, 24, Color::new(55, 74, 96, 255));
        d.draw_text("- One extra air jump", 300, 292, 24, Color::new(55, 74, 96, 255));
        d.draw_text("SPACE / UP = jump   R = restart run", 230, 346, 24, Color::new(70, 86, 112, 255));
        d.draw_text("M toggles sound effects   B toggles music", 220, 376, 22, Color::new(70, 86, 112, 255));
        d.draw_text("Press ENTER or SPACE to start", 270, 420, 28, Color::new(181, 76, 76, 255));
    }

    fn draw_game_over_overlay(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle(210, 140, 540, 240, Color::new(20, 26, 38, 190));
        d.draw_text("Run Over!", 372, 176, 40, Color::new(255, 222, 120, 255));
        d.draw_text(&format!("Final score: {}", self.total_score()), 344, 230, 28, Color::WHITE);
        d.draw_text(&format!("Best score: {}", self.best_score), 350, 266, 24, Color::new(220, 231, 248, 255));
        d.draw_text("Press R or ENTER to play again", 276, 314, 24, Color::new(255, 255, 255, 255));
        d.draw_text("Press ESC to quit   M sound   B music", 300, 344, 20, Color::new(210, 219, 234, 255));
    }
}

fn circle_rect_collision(cx: f32, cy: f32, radius: f32, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
    let closest_x = cx.clamp(rx, rx + rw);
    let closest_y = cy.clamp(ry, ry + rh);
    let dx = cx - closest_x;
    let dy = cy - closest_y;

    dx * dx + dy * dy <= radius * radius
}

fn circle_triangle_collision(cx: f32, cy: f32, radius: f32, a: Vector2, b: Vector2, c: Vector2) -> bool {
    let center = Vector2::new(cx, cy);
    point_in_triangle(center, a, b, c)
        || distance_to_segment(center, a, b) <= radius
        || distance_to_segment(center, b, c) <= radius
        || distance_to_segment(center, c, a) <= radius
}

fn distance_to_segment(point: Vector2, start: Vector2, end: Vector2) -> f32 {
    let segment_x = end.x - start.x;
    let segment_y = end.y - start.y;
    let length_sq = segment_x * segment_x + segment_y * segment_y;

    if length_sq <= f32::EPSILON {
        let dx = point.x - start.x;
        let dy = point.y - start.y;
        return (dx * dx + dy * dy).sqrt();
    }

    let t = (((point.x - start.x) * segment_x + (point.y - start.y) * segment_y) / length_sq).clamp(0.0, 1.0);
    let projection_x = start.x + t * segment_x;
    let projection_y = start.y + t * segment_y;
    let dx = point.x - projection_x;
    let dy = point.y - projection_y;
    (dx * dx + dy * dy).sqrt()
}

fn point_in_triangle(p: Vector2, a: Vector2, b: Vector2, c: Vector2) -> bool {
    let area = 0.5 * (-b.y * c.x + a.y * (-b.x + c.x) + a.x * (b.y - c.y) + b.x * c.y);
    let s = 1.0 / (2.0 * area) * (a.y * c.x - a.x * c.y + (c.y - a.y) * p.x + (a.x - c.x) * p.y);
    let t = 1.0 / (2.0 * area) * (a.x * b.y - a.y * b.x + (a.y - b.y) * p.x + (b.x - a.x) * p.y);
    s >= 0.0 && t >= 0.0 && (1.0 - s - t) >= 0.0
}
