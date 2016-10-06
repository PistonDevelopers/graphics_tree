//! A 2D graphics backend for Piston-Graphics that stores and optimizes commands.

#![deny(missing_docs)]

extern crate graphics;
extern crate image;
extern crate range;

use std::sync::{Arc, RwLock};

use graphics::{Context, DrawState, Graphics, ImageSize};
use graphics::types::Color;
use image::RgbaImage;
use range::Range;

/// A graphics backend that stores and optimizes commands
pub struct GraphicsTree {
    commands: Vec<Command>,
    vertices: Vec<f32>,
    uvs: Vec<f32>,
    current_color: Color,
    current_draw_state: DrawState,
}

enum Command {
    ClearColor(Color),
    ClearStencil(u8),
    ChangeColor(Color),
    ChangeDrawState(DrawState),
    Colored(Range),
}

/// Simplifies some common operations on textures.
pub struct Texture(pub Arc<RwLock<TextureInner>>);

/// Stores the inner data to keep track of a texture.
pub struct TextureInner {
    /// Id used by `TextureBuffer` to look up texture.
    pub id: Option<u32>,
    /// Whether the texture needs to be updated.
    pub needs_update: bool,
    /// The image data associated with a texture.
    pub image: RgbaImage,
}

impl GraphicsTree {
    /// Creates a new graphics tree.
    pub fn new() -> GraphicsTree {
        GraphicsTree {
            commands: vec![],
            vertices: vec![],
            uvs: vec![],
            current_color: [0.0; 4],
            current_draw_state: Default::default(),
        }
    }

    /// Returns `true` if graphics tree is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.len() == 0 &&
        self.vertices.len() == 0 &&
        self.uvs.len() == 0
    }

    /// Clears all graphics.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.vertices.clear();
        self.uvs.clear();
    }

    /// Draws graphics to backend.
    pub fn draw<G>(&self, c: &Context, g: &mut G)
        where G: Graphics
    {
        use Command::*;
        use graphics::BACK_END_MAX_VERTEX_COUNT;

        let bufsize = 2 * BACK_END_MAX_VERTEX_COUNT;
        let mut color: Color = [0.0; 4];
        let mut draw_state: DrawState = Default::default();
        for command in &self.commands {
            match *command {
                ClearColor(color) => g.clear_color(color),
                ClearStencil(value) => g.clear_stencil(value),
                ChangeColor(new_color) => color = new_color,
                ChangeDrawState(new_draw_state) => draw_state = new_draw_state,
                Colored(range) => {
                    // Split range in chunks to respect `Graphics` interface.
                    let offset = range.offset;
                    let length = range.length;
                    let chunks = length / bufsize;
                    g.tri_list(&draw_state, &color, |mut f| {
                        for i in 0..chunks {
                            let start = offset + chunks * i;
                            let end = start + bufsize;
                            f(&self.vertices[start..end]);
                        }
                        if chunks * bufsize < length {
                            let start = chunks * bufsize;
                            let len = length - start;
                            f(&self.vertices[offset + start..offset + len]);
                        }
                    });
                }
            }
        }
    }
}

impl ImageSize for Texture {
    fn get_size(&self) -> (u32, u32) {
        use std::ops::Deref;

        self.0.read().unwrap().deref().image.dimensions()
    }
}

impl Graphics for GraphicsTree {
    type Texture = Texture;

    fn clear_color(&mut self, color: Color) {
        self.commands.push(Command::ClearColor(color));
    }

    fn clear_stencil(&mut self, value: u8) {
        self.commands.push(Command::ClearStencil(value));
    }

    fn tri_list<F>(
        &mut self,
        draw_state: &DrawState,
        color: &Color,
        mut f: F
    ) where F: FnMut(&mut FnMut(&[f32])) {
        if color != &self.current_color {
            self.commands.push(Command::ChangeColor(*color));
        }
        if draw_state != &self.current_draw_state {
            self.commands.push(Command::ChangeDrawState(*draw_state));
        }
        let start = self.vertices.len();
        f(&mut |chunk| self.vertices.extend_from_slice(chunk));
        self.commands.push(Command::Colored(Range::new(start, self.vertices.len() - start)));
    }

    fn tri_list_uv<F>(
        &mut self,
        draw_state: &DrawState,
        color: &[f32; 4],
        texture: &Self::Texture,
        f: F
    ) where F: FnMut(&mut FnMut(&[f32], &[f32])) {

    }
}
