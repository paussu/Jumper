use raylib::prelude::*;
use rand::Rng;

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 450;
const BALL_RADIUS: f32 = 20.0;
const GRAVITY: f32 = 0.5;
const JUMP_VELOCITY: f32 = -12.0;
const OBSTACLE_WIDTH: i32 = 30;
const OBSTACLE_HEIGHT: i32 = 50;

enum ObstacleShape {
    Rectangle,
    Triangle,
}

struct Obstacle {
    x: i32,
    y: i32,
    speed: i32,
    shape: ObstacleShape,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Ball jumper with raylib")
        .build();
    let mut rng = rand::thread_rng();

    rl.set_target_fps(60);

    let mut ball_y = SCREEN_HEIGHT as f32 - BALL_RADIUS;
    let mut velocity = 0.0;
    let mut obstacles: Vec<Obstacle> = vec![];
    let mut obstacle: Option<Obstacle> = None;
    let mut frame_count = 0;

    let mut game_over = false;

    while !rl.window_should_close() && !game_over {
        // Jump input
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) && ball_y >= SCREEN_HEIGHT as f32 - BALL_RADIUS {
            velocity = JUMP_VELOCITY;
        }

        // Physics
        velocity += GRAVITY;
        ball_y += velocity;
        if ball_y > SCREEN_HEIGHT as f32 - BALL_RADIUS {
            ball_y = SCREEN_HEIGHT as f32 - BALL_RADIUS;
            velocity = 0.0;
        }

        // Spawn a new obstacle only if none exists
        if obstacle.is_none() {
            let shape = if rand::random() {
                ObstacleShape::Rectangle
            } else {
                ObstacleShape::Triangle
            };
            obstacle = Some(Obstacle {
                x: SCREEN_WIDTH,
                y: SCREEN_HEIGHT - OBSTACLE_HEIGHT,
                speed: rng.gen_range(5..15),
                shape,
            });
        }


        // Move obstacle
        if let Some(obs) = &mut obstacle {
            obs.x -= obs.speed;

            // Remove obstacle if it moves off-screen
            if obs.x + OBSTACLE_WIDTH < 0 {
                obstacle = None;
            }
        }

        // Collision detection
        if let Some(obs) = &obstacle {
            let ball_rect = Rectangle {
                x: 100.0 - BALL_RADIUS,
                y: ball_y - BALL_RADIUS,
                width: BALL_RADIUS * 2.0,
                height: BALL_RADIUS * 2.0,
            };
            let obs_rect = Rectangle {
                x: obs.x as f32,
                y: obs.y as f32,
                width: OBSTACLE_WIDTH as f32,
                height: OBSTACLE_HEIGHT as f32,
            };
            match obs.shape {
                ObstacleShape::Rectangle => {
                    if ball_rect.check_collision_recs(&obs_rect) {
                        game_over = true;
                    }
                }
                ObstacleShape::Triangle => {
                    let ball_center = Vector2::new(100.0, ball_y);
                    let p1 = Vector2::new(obs.x as f32, obs.y as f32 + OBSTACLE_HEIGHT as f32);
                    let p2 = Vector2::new(obs.x as f32 + OBSTACLE_WIDTH as f32 / 2.0, obs.y as f32);
                    let p3 = Vector2::new(obs.x as f32 + OBSTACLE_WIDTH as f32, obs.y as f32 + OBSTACLE_HEIGHT as f32);
                    if point_in_triangle(ball_center, p1, p2, p3) {
                        game_over = true;
                    }
                }
            }
        }

        // Drawing
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::SKYBLUE);

        d.draw_circle(100, ball_y as i32, BALL_RADIUS, Color::BLACK);
        d.draw_circle(100, ball_y as i32, BALL_RADIUS - 2.0, Color::RED);

        if let Some(obs) = &obstacle {
            match obs.shape {
                ObstacleShape::Rectangle => {
                    d.draw_rectangle(obs.x, obs.y, OBSTACLE_WIDTH, OBSTACLE_HEIGHT, Color::BLACK);
                    d.draw_rectangle(obs.x + 5, obs.y + 5, OBSTACLE_WIDTH - 10, OBSTACLE_HEIGHT - 5, Color::GRAY);
                }
                ObstacleShape::Triangle => {
                    let p1 = Vector2::new(obs.x as f32, obs.y as f32 + OBSTACLE_HEIGHT as f32);
                    let p2 = Vector2::new(obs.x as f32 + OBSTACLE_WIDTH as f32 / 2.0, obs.y as f32);
                    let p3 = Vector2::new(obs.x as f32 + OBSTACLE_WIDTH as f32, obs.y as f32 + OBSTACLE_HEIGHT as f32);
                    d.draw_triangle(p2, p1, p3, Color::BLACK);
                    d.draw_triangle(
                        Vector2::new(p2.x, p2.y + 5.0),
                        Vector2::new(p1.x + 5.0, p1.y - 5.0),
                        Vector2::new(p3.x - 5.0, p3.y - 5.0),
                        Color::RED,
                    );
                }
            }
        }

        d.draw_text("Press SPACE to jump", 10, 10, 20, Color::BLACK);
    }

    // Game over loop
    if game_over {
        wait_for_exit(&mut rl, &thread);
    }
}

fn point_in_triangle(p: Vector2, a: Vector2, b: Vector2, c: Vector2) -> bool {
    let area = 0.5 * (-b.y * c.x + a.y * (-b.x + c.x) + a.x * (b.y - c.y) + b.x * c.y);
    let s = 1.0 / (2.0 * area) * (a.y * c.x - a.x * c.y + (c.y - a.y) * p.x + (a.x - c.x) * p.y);
    let t = 1.0 / (2.0 * area) * (a.x * b.y - a.y * b.x + (a.y - b.y) * p.x + (b.x - a.x) * p.y);
    s > 0.0 && t > 0.0 && (1.0 - s - t) > 0.0
}

fn wait_for_exit(rl: &mut RaylibHandle, thread: &RaylibThread) {
    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            break;
        }

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::SKYBLUE);
        d.draw_text("Game Over!", SCREEN_WIDTH / 2 - 80, SCREEN_HEIGHT / 2 - 20, 30, Color::RED);
        d.draw_text("Press ENTER or ESC to exit", SCREEN_WIDTH / 2 - 120, SCREEN_HEIGHT / 2 + 20, 20, Color::DARKGRAY);
    }
}