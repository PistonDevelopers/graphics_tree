extern crate piston_window;
extern crate image as im;
extern crate vecmath;
extern crate graphics_tree;

use piston_window::*;
use vecmath::{vec2_len, vec2_sub};
use graphics_tree::{GraphicsTree, TextureBuffer};

fn main() {
    let opengl = OpenGL::V3_2;
    let (width, height) = (300, 300);
    let mut window: PistonWindow =
        WindowSettings::new("graphics_tree: paint", (width, height))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let ref mut graphics_tree = GraphicsTree::new();
    let ref mut texture_buffer = TextureBuffer::new(TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    });

    let canvas = im::ImageBuffer::new(width, height).into();
    let mut draw = false;
    let mut last_pos = None;

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, _| {
            if graphics_tree.is_empty() {
                clear([1.0; 4], graphics_tree);
                image(&canvas, c.transform, graphics_tree);
            }

            graphics_tree.draw(texture_buffer, g);
        });
        if let Some(button) = e.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                draw = true;
                last_pos = e.mouse_cursor_args()
            }
        };
        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                draw = false;
                last_pos = None
            }
        };
        if draw {
            if let Some(pos) = e.mouse_cursor_args() {
                let (x, y) = (pos[0] as f32, pos[1] as f32);

                if let Some(p) = last_pos {
                    canvas.with_image_mut(|canvas| {
                        let (last_x, last_y) = (p[0] as f32, p[1] as f32);
                        let distance = vec2_len(vec2_sub(p, pos)) as u32;

                        for i in 0..distance {
                            let diff_x = x - last_x;
                            let diff_y = y - last_y;
                            let delta = i as f32 / distance as f32;
                            let new_x = (last_x + (diff_x * delta)) as u32;
                            let new_y = (last_y + (diff_y * delta)) as u32;
                            if new_x < width && new_y < height {
                                canvas.put_pixel(new_x, new_y, im::Rgba([0, 0, 0, 255]));
                            };
                        };
                    });
                    graphics_tree.clear();
                };

                last_pos = Some(pos)
            };

        }
    }
}
