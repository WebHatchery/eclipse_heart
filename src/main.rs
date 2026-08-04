use eclipse_heart::game::Game;
use eclipse_heart::ui;
use macroquad::prelude::*;
use macroquad_toolkit::capture;

fn window_conf() -> Conf {
    // Built by hand (not capture::capture_window_conf) to keep the game's
    // large default window size (2560x1440) while still forcing windowed
    // mode during capture so screenshots are deterministic.
    // Hand-built Conf means no automatic arming: without this the capture run
    // puts a full game window on the desktop for its whole duration.
    capture::headless::arm("ECLIPSE_HEART");
    let capture_mode = capture::capture_requested("ECLIPSE_HEART");
    Conf {
        window_title: "Eclipse Heart".to_owned(),
        window_width: capture::env_i32("ECLIPSE_HEART_WINDOW_WIDTH", 2560),
        window_height: capture::env_i32("ECLIPSE_HEART_WINDOW_HEIGHT", 1440),
        window_resizable: true,
        fullscreen: capture::env_bool("ECLIPSE_HEART_FULLSCREEN", !capture_mode),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: when ECLIPSE_HEART_CAPTURE_PATH is set, seed a
    // scene, simulate deterministic frames, write a PNG, and exit.
    if let Some(mut config) = capture::CaptureConfig::from_env("ECLIPSE_HEART") {
        config.frames = capture::env_u32("ECLIPSE_HEART_CAPTURE_FRAMES", 8).max(1);
        assert!(
            game.prepare_capture_screen(&config.scene),
            "unknown capture screen: {}",
            config.scene
        );
        capture::run_capture(&config, |_dt| {
            show_mouse(true);
            clear_background(ui::core::BACKGROUND);
            game.update_for_capture();
            game.draw();
        })
        .await;
        return;
    }

    loop {
        show_mouse(true);
        clear_background(ui::core::BACKGROUND);
        game.update();
        game.draw();
        next_frame().await;
    }
}
