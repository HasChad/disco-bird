#![windows_subsystem = "windows"]

use macroquad::experimental::animation::*;
use macroquad::{
    audio::{play_sound, play_sound_once, PlaySoundParams},
    prelude::*,
};
use miniquad::conf::Icon;
use rust_embed::{Embed, EmbeddedFile};
use std::f32::consts::PI;

const BIRD_SIZE: f32 = 17.0;
const PIPE_GAP: f32 = 170.0;
const PIPE_DISTANCE: f32 = 250.0;
const GRAVITY: f32 = 2000.0;
const JUMP_FORCE: f32 = 600.0;
const GAME_SPEED: f32 = 150.0;
const PIPE1_START: f32 = 432. + 100.; //screen width + 100.

#[derive(Embed)]
#[folder = "assets"]
struct Asset;

#[derive(PartialEq)]
enum GameState {
    Start,
    Gameplay,
    GameOver,
}

struct Bird {
    x: f32,
    y: f32,
    vy: f32,
}

struct Pipe {
    x: f32,
    y: f32,
}

struct Ground {
    x: f32,
    y: f32,
}

fn populate_array(img: Image, array: &mut [u8]) {
    let mut index: usize = 0;
    for pixel in img.get_image_data() {
        for value in pixel.iter() {
            array[index] = *value;
            index += 1;
        }
    }
}

fn icon_formatter() -> Icon {
    let mut array_small: [u8; 1024] = [0; 1024];
    let mut array_medium: [u8; 4096] = [0; 4096];
    let mut array_big: [u8; 16384] = [0; 16384];

    populate_array(
        Image::from_file_with_format(
            &Asset::get("icons/cool-icon16.png").unwrap().data,
            Some(ImageFormat::Png),
        )
        .unwrap(),
        &mut array_small,
    );
    populate_array(
        Image::from_file_with_format(
            &Asset::get("icons/cool-icon32.png").unwrap().data,
            Some(ImageFormat::Png),
        )
        .unwrap(),
        &mut array_medium,
    );
    populate_array(
        Image::from_file_with_format(
            &Asset::get("icons/cool-icon64.png").unwrap().data,
            Some(ImageFormat::Png),
        )
        .unwrap(),
        &mut array_big,
    );

    Icon {
        small: array_small,
        medium: array_medium,
        big: array_big,
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "DiscoBird".into(),
        icon: Some(icon_formatter()),
        window_width: 432,
        window_height: 768,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    // texture load
    let bird_texture = Texture2D::from_file_with_format(
        &Asset::get("sprites/cool-bird.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let pipe_texture = Texture2D::from_file_with_format(
        &Asset::get("sprites/cool-pipe.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let ground_texture = Texture2D::from_file_with_format(
        &Asset::get("sprites/cool-ground.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let bg_texture = Texture2D::from_file_with_format(
        &Asset::get("sprites/cool-bg.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let mut bird_sprite = AnimatedSprite::new(
        60,
        36,
        &[Animation {
            name: "fly".to_string(),
            row: 0,
            frames: 3,
            fps: 12,
        }],
        true,
    );

    // text texture load
    let game_over_text = Texture2D::from_file_with_format(
        &Asset::get("sprites/game-over-text.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let start_info_text = Texture2D::from_file_with_format(
        &Asset::get("sprites/start-info-text.png").unwrap().data,
        Some(ImageFormat::Png),
    );
    let game_name_text = Texture2D::from_file_with_format(
        &Asset::get("sprites/disco-bird-text.png").unwrap().data,
        Some(ImageFormat::Png),
    );

    // sound load
    let point_sound =
        macroquad::audio::load_sound_from_bytes(&Asset::get("sounds/point.wav").unwrap().data)
            .await
            .unwrap();
    let smack_sound =
        macroquad::audio::load_sound_from_bytes(&Asset::get("sounds/smack.wav").unwrap().data)
            .await
            .unwrap();
    let jump_sound =
        macroquad::audio::load_sound_from_bytes(&Asset::get("sounds/hnh.wav").unwrap().data)
            .await
            .unwrap();
    let death_sound =
        macroquad::audio::load_sound_from_bytes(&Asset::get("sounds/death.wav").unwrap().data)
            .await
            .unwrap();
    let bg_music =
        macroquad::audio::load_sound_from_bytes(&Asset::get("sounds/bg-music.wav").unwrap().data)
            .await
            .unwrap();

    // initialization
    let mut game_score = 0;
    let mut high_score = 0;

    let mut game_state = GameState::Start;
    play_sound(
        &bg_music,
        PlaySoundParams {
            looped: true,
            volume: 1.0,
        },
    );

    let mut bird = Bird {
        x: screen_width() / 2.,
        y: screen_height() / 2. - 100.,
        vy: 0.0,
    };

    let mut ground1 = Ground {
        x: 0.,
        y: screen_height() - ground_texture.height(),
    };

    let mut ground2 = Ground {
        x: ground_texture.width(),
        y: screen_height() - ground_texture.height(),
    };

    let mut pipe1 = Pipe {
        x: PIPE1_START,
        y: rand::gen_range(240., 560.),
    };

    let mut pipe2 = Pipe {
        x: pipe1.x + PIPE_DISTANCE,
        y: rand::gen_range(240., 560.),
    };

    // Game loop
    loop {
        if game_state == GameState::Start || game_state == GameState::Gameplay {
            // handle input
            if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
                play_sound_once(&jump_sound);
                bird.vy = -JUMP_FORCE;
                game_state = GameState::Gameplay;
            }

            // update grounds
            if ground1.x <= 0. && ground1.x > -ground_texture.width() {
                ground1.x -= GAME_SPEED * get_frame_time();
                ground2.x = ground1.x + ground_texture.width();
            }

            if ground2.x <= 0. && ground2.x > -ground_texture.width() {
                ground2.x -= GAME_SPEED * get_frame_time();
                ground1.x = ground2.x + ground_texture.width();
            }
        }
        if game_state == GameState::Gameplay {
            // update bird
            bird.y += bird.vy * get_frame_time();
            bird.vy += GRAVITY * get_frame_time();

            // update pipes
            pipe1.x -= GAME_SPEED * get_frame_time();
            pipe2.x -= GAME_SPEED * get_frame_time();

            if pipe1.x + pipe_texture.width() < 0. {
                pipe1.x = pipe2.x + PIPE_DISTANCE;
                pipe1.y = rand::gen_range(240., 560.);
            }

            if pipe2.x + pipe_texture.width() < 0. {
                pipe2.x = pipe1.x + PIPE_DISTANCE;
                pipe2.y = rand::gen_range(240., 560.);
            }

            // update game score
            if bird.x - 1. < pipe1.x + pipe_texture.width()
                && bird.x + 1. > pipe1.x + pipe_texture.width()
                && game_score % 2 == 0
            {
                game_score += 1;
                play_sound_once(&point_sound);
            }

            if bird.x - 1. < pipe2.x + pipe_texture.width()
                && bird.x + 1. > pipe2.x + pipe_texture.width()
                && game_score % 2 == 1
            {
                game_score += 1;
                play_sound_once(&point_sound);
            }

            // check collisions
            if pipe1.x < bird.x + BIRD_SIZE
                && pipe1.x + pipe_texture.width() > bird.x - BIRD_SIZE
                && !(pipe1.y > bird.y + BIRD_SIZE && pipe1.y - PIPE_GAP < bird.y - BIRD_SIZE)
            {
                play_sound_once(&death_sound);
                play_sound_once(&smack_sound);
                game_state = GameState::GameOver;
            }

            if pipe2.x < bird.x + BIRD_SIZE
                && pipe2.x + pipe_texture.width() > bird.x - BIRD_SIZE
                && !(pipe2.y > bird.y + BIRD_SIZE && pipe2.y - PIPE_GAP < bird.y - BIRD_SIZE)
            {
                play_sound_once(&death_sound);
                play_sound_once(&smack_sound);
                game_state = GameState::GameOver;
            }

            if bird.y - BIRD_SIZE < 0.
                || bird.y + BIRD_SIZE > screen_height() - ground_texture.height()
            {
                play_sound_once(&death_sound);
                play_sound_once(&smack_sound);
                game_state = GameState::GameOver;
            }
        }

        if game_state == GameState::GameOver {
            if bird.y + BIRD_SIZE < ground1.y || bird.y + BIRD_SIZE < ground2.y {
                bird.y += bird.vy * get_frame_time();
                bird.vy += GRAVITY * get_frame_time();
            } else if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
                game_state = GameState::Start;
                game_score = 0;

                bird = Bird {
                    x: screen_width() / 2.,
                    y: screen_height() / 2. - 100.,
                    vy: 0.0,
                };

                pipe1 = Pipe {
                    x: PIPE1_START,
                    y: rand::gen_range(240., 560.),
                };

                pipe2 = Pipe {
                    x: pipe1.x + PIPE_DISTANCE,
                    y: rand::gen_range(240., 560.),
                };
            }
        }

        // ! ==> MAIN RENDERING <==
        clear_background(WHITE);

        draw_texture(&bg_texture, 0., 0., WHITE);

        // lower pipe
        draw_texture(&pipe_texture, pipe1.x, pipe1.y, WHITE);
        // upper pipe
        draw_texture_ex(
            &pipe_texture,
            pipe1.x,
            pipe1.y - pipe_texture.height() - PIPE_GAP,
            WHITE,
            DrawTextureParams {
                flip_y: true,
                ..Default::default()
            },
        );

        // lower pipe
        draw_texture(&pipe_texture, pipe2.x, pipe2.y, WHITE);
        // upper pipe
        draw_texture_ex(
            &pipe_texture,
            pipe2.x,
            pipe2.y - pipe_texture.height() - PIPE_GAP,
            WHITE,
            DrawTextureParams {
                flip_y: true,
                ..Default::default()
            },
        );

        let bird_rotation = {
            if bird.vy > 1000. {
                90.
            } else if bird.vy >= 0. {
                bird.vy / 1000. * 90.
            } else {
                bird.vy / -500. * -30.
            }
        };

        // bird animation
        draw_texture_ex(
            &bird_texture,
            bird.x - bird_texture.width() / 2. + 65.,
            bird.y - bird_texture.height() / 2.,
            WHITE,
            DrawTextureParams {
                source: Some(bird_sprite.frame().source_rect),
                dest_size: Some(bird_sprite.frame().dest_size),
                rotation: bird_rotation / 180. * PI,
                ..Default::default()
            },
        );

        if game_state == GameState::Start {
            draw_texture(
                &game_name_text,
                screen_width() / 2. - game_name_text.width() / 2.,
                screen_height() / 2. - 200.,
                WHITE,
            );

            draw_texture(
                &start_info_text,
                screen_width() / 2. - start_info_text.width() / 2.,
                screen_height() / 2. - 70.,
                WHITE,
            );

            // update frame
            bird_sprite.update();
        }

        if game_state == GameState::Gameplay {
            // game score
            let game_score_text = String::from(game_score.to_string().as_str());
            let font_size = 75.;
            let text_size = measure_text(&game_score_text, None, font_size as _, 1.0);

            draw_text(
                &game_score_text,
                screen_width() / 2. - text_size.width / 2.,
                text_size.height / 2. + 30.,
                font_size,
                BLACK,
            );

            // update frame
            bird_sprite.update();
        }

        if game_state == GameState::GameOver {
            draw_texture(
                &game_over_text,
                screen_width() / 2. - game_over_text.width() / 2.,
                screen_height() / 2. - 200.,
                WHITE,
            );

            if game_score > high_score {
                high_score = game_score
            }

            // game score
            let mut text = String::from(format!("Score: {}", game_score).as_str());
            let font_size = 60.;
            let mut text_size = measure_text(&text, None, font_size as _, 1.0);

            draw_text(
                &text,
                screen_width() / 2. - text_size.width / 2.,
                screen_height() / 2. - text_size.height / 2. + font_size - 130.,
                font_size,
                BLACK,
            );

            // high score
            text = String::from(format!("High Score: {}", high_score).as_str());
            text_size = measure_text(&text, None, font_size as _, 1.0);

            draw_text(
                &text,
                screen_width() / 2. - text_size.width / 2.,
                screen_height() / 2. - text_size.height / 2. + font_size - 80.,
                font_size,
                BLACK,
            );
        }

        draw_texture(&ground_texture, ground1.x, ground1.y, WHITE);
        draw_texture(&ground_texture, ground2.x, ground2.y, WHITE);

        next_frame().await;
    }
}
