mod app;
mod audio;
mod domain;
mod input;
mod render;

use app::GameApp;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Centipede 1981".to_owned(),
        window_width: 960,
        window_height: 1024,
        high_dpi: false,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = GameApp::new().await;

    loop {
        if app.frame() {
            break;
        }
        next_frame().await;
    }
}
