use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Frogger".to_string(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut world = hecs::World::new();
    
    let a = world.spawn((123, true, "abc"));
    let b = world.spawn((42, false));
    
    for (number, &flag) in world.query_mut::<(&mut i32, &bool)>() {
    if flag { *number *= 2; }
    }
    
    assert_eq!(*world.get::<&i32>(a).unwrap(), 246);
    assert_eq!(*world.get::<&i32>(b).unwrap(), 42);
    
    loop {
        clear_background(RED);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

        draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}