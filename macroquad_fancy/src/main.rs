use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Macroquad Fancy".to_string(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    println!("Hello");
}
