use macroquad::prelude::*;

mod demo;
mod palette;
mod postfx;
mod scene;
mod scenes;
mod shaders;
mod transitions;

fn window_conf() -> Conf {
    Conf {
        window_title: "macroquad_fancy".to_string(),
        window_width: 1600,
        window_height: 900,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut runner = demo::DemoRunner::new();

    loop {
        let dt = get_frame_time();
        runner.update(dt);
        runner.draw();
        next_frame().await;
    }
}
