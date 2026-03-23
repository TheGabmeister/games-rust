use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Macroquad Fancy".to_string(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    println!("Hello");
}
