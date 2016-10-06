extern crate piston_window;
extern crate graphics_tree;
extern crate image as im;

use piston_window::*;
use graphics_tree::{GraphicsTree, TextureBuffer};

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("graphics_tree: rectangle_fps", [512; 2])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let ref mut graphics_tree = GraphicsTree::new();

    let img = im::open("assets/rust.png").unwrap();
    let tex = match img {
        im::DynamicImage::ImageRgba8(img) => img,
        img => img.to_rgba()
    }.into();

    let ref mut tx_buffer = TextureBuffer::new(window.factory.clone());

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            if graphics_tree.is_empty() {
                clear([1.0; 4], graphics_tree);
                image(&tex, c.transform, graphics_tree);
            }

            graphics_tree.draw(tx_buffer, g);
        });
    }
}
