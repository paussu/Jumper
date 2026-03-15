mod audio;
mod game;

use raylib::prelude::*;

use crate::audio::{AudioFx, BackgroundMusic};
use crate::game::{Game, Scene, SCREEN_HEIGHT, SCREEN_WIDTH};

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Jumper")
        .build();
    rl.set_target_fps(60);

    let audio = RaylibAudio::init_audio_device().expect("failed to initialize audio device");
    audio.set_master_volume(0.7);
    let mut sound_fx = AudioFx::new(&audio).expect("failed to build sound effects");
    let mut background_music = BackgroundMusic::new(&audio).expect("failed to build background music");

    let mut rng = rand::rng();
    let mut game = Game::new(&mut rng);

    while !rl.window_should_close() {
        background_music.update();

        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            sound_fx.toggle_mute();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            background_music.toggle_mute();
        }

        match game.scene() {
            Scene::Title => {
                game.update_title(&mut rng);
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    game.reset_run(&mut rng);
                    sound_fx.play_start();
                }
            }
            Scene::Playing => {
                if rl.is_key_pressed(KeyboardKey::KEY_R) {
                    game.reset_run(&mut rng);
                    sound_fx.play_start();
                } else {
                    game.update_playing(&rl, &mut rng, &sound_fx);
                }
            }
            Scene::GameOver => {
                game.update_game_over(&mut rng);

                if rl.is_key_pressed(KeyboardKey::KEY_R) || rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    game.reset_run(&mut rng);
                    sound_fx.play_start();
                }
            }
        }

        let mut d = rl.begin_drawing(&thread);
        game.draw(&mut d, &sound_fx, &background_music);
    }
}
