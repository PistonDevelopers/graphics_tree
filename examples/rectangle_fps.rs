extern crate piston_window;
extern crate graphics_tree;
extern crate rand;
extern crate fps_counter;

use piston_window::*;
use graphics_tree::{GraphicsTree, TextureBuffer};
use rand::{thread_rng, Rng};
use fps_counter::FPSCounter;

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("graphics_tree: rectangle_fps", [512; 2])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let ref mut graphics_tree = GraphicsTree::new();
    let ref mut texture_buffer = TextureBuffer::new(TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    });

    let mut fps_counter = FPSCounter::new();
    let mut fps = 0;
    let n = std::env::args_os().nth(1)
        .and_then(|s| s.into_string().ok())
        .and_then(|n| n.parse().ok())
        .unwrap_or(17_000);;

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, _| {
            if graphics_tree.is_empty() {
                draw(n, &c, graphics_tree);
            }

            graphics_tree.draw(texture_buffer, g);
            fps = fps_counter.tick();
        });
        window.set_title(fps.to_string());
    }

    println!("{}", fps);
}

fn draw<G: Graphics>(n: u32, c: &Context, g: &mut G) {
    clear([1.0; 4], g);

    let mut rng = thread_rng();
    for _ in 0..n {
        let x: f64 = rng.gen::<f64>() * 512.0;
        let y: f64 = rng.gen::<f64>() * 512.0;
        rectangle([1.0, 0.0, 1.0, 1.0],
            [x, y, 10.0, 2.0],
            c.transform, g);
    }
}
