//! A 2D graphics backend for Piston-Graphics that stores and optimizes commands.

#![deny(missing_docs)]

extern crate graphics;
extern crate image;
extern crate range;
extern crate texture;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use graphics::{DrawState, Graphics, ImageSize};
use graphics::types::Color;
use image::RgbaImage;
use range::Range;
use texture::CreateTexture;

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
    Textured(Texture, Range, Range),
}

/// Simplifies some common operations on textures.
#[derive(Clone)]
pub struct Texture(pub Arc<RwLock<TextureInner>>);

/// Stores the inner data to keep track of a texture.
pub struct TextureInner {
    /// Id used by `TextureBuffer` to look up texture.
    pub id: Option<u64>,
    /// Whether the texture needs to be updated.
    pub needs_update: bool,
    /// The image data associated with a texture.
    pub image: RgbaImage,
}

/// Stores textures.
pub struct TextureBuffer<F, T> {
    /// The factory that creates textures.
    pub factory: F,
    textures: HashMap<u64, T>,
    next_id: u64,
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
    pub fn draw<F, T, G>(
        &self,
        texture_buffer: &mut TextureBuffer<F, T>,
        g: &mut G
    )
        where
            T: ImageSize + CreateTexture<F>,
            G: Graphics<Texture=T>
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
                Textured(ref tex, vertex_range, uv_range) => {
                    // Split range in chunks to respect `Graphics` interface.
                    let offset_v = vertex_range.offset;
                    let length_v = vertex_range.length;
                    let chunks_v = length_v / bufsize;
                    let offset_uv = uv_range.offset;
                    let length_uv = uv_range.length;
                    let chunks_uv = length_uv / bufsize;

                    let texture = if let Ok(mut inner) = tex.0.write() {
                        if inner.id.is_none() {
                            use texture::{Format, TextureSettings};

                            let (width, height) = inner.image.dimensions();
                            let new_texture: T = CreateTexture::create(
                                &mut texture_buffer.factory,
                                Format::Rgba8,
                                &inner.image,
                                [width, height],
                                &TextureSettings::new()
                            ).unwrap_or_else(|_| panic!("Could not create texture"));
                            texture_buffer.textures.insert(texture_buffer.next_id, new_texture);
                            inner.id = Some(texture_buffer.next_id);
                            texture_buffer.next_id += 1;
                        } else if inner.needs_update {
                            // Create a new texture, because updating is not
                            // supported directly yet.
                            use texture::{Format, TextureSettings};

                            let id = inner.id.unwrap();
                            let (width, height) = inner.image.dimensions();
                            let new_texture: T = CreateTexture::create(
                                &mut texture_buffer.factory,
                                Format::Rgba8,
                                &inner.image,
                                [width, height],
                                &TextureSettings::new()
                            ).unwrap_or_else(|_| panic!("Could not create texture"));
                            texture_buffer.textures.insert(id, new_texture);
                            inner.needs_update = false;
                        }
                        if let Some(texture) = texture_buffer.textures.get(&inner.id.unwrap()) {
                            texture
                        } else {
                            panic!("Texture does not exist");
                        }
                    } else {
                        panic!("Image is used elsewhere");
                    };

                    g.tri_list_uv(&draw_state, &color, texture, |mut f| {
                        for i in 0..chunks_v {
                            let start_v = offset_v + chunks_v * i;
                            let end_v = start_v + bufsize;
                            let start_uv = offset_uv + chunks_uv * i;
                            let end_uv = start_uv + bufsize;
                            f(&self.vertices[start_v..end_v],
                              &self.uvs[start_uv..end_uv]);
                        }
                        if chunks_v * bufsize < length_v {
                            let start_v = chunks_v * bufsize;
                            let len_v = length_v - start_v;
                            let start_uv = chunks_uv * bufsize;
                            let len_uv = length_uv - start_uv;
                            f(&self.vertices[offset_v + start_v..offset_v + len_v],
                              &self.uvs[offset_uv + start_uv..offset_uv + len_uv]);
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
        mut f: F
    ) where F: FnMut(&mut FnMut(&[f32], &[f32])) {
        if color != &self.current_color {
            self.commands.push(Command::ChangeColor(*color));
        }
        if draw_state != &self.current_draw_state {
            self.commands.push(Command::ChangeDrawState(*draw_state));
        }
        let start_vertices = self.vertices.len();
        let start_uvs = self.uvs.len();
        f(&mut |chunk, chunk_uvs| {
            self.vertices.extend_from_slice(chunk);
            self.uvs.extend_from_slice(chunk_uvs);
        });
        self.commands.push(Command::Textured(
            texture.clone(),
            Range::new(start_vertices, self.vertices.len() - start_vertices),
            Range::new(start_uvs, self.uvs.len() - start_uvs)
        ));
    }
}

impl From<RgbaImage> for Texture {
    fn from(image: RgbaImage) -> Texture {
        Texture(Arc::new(RwLock::new(TextureInner {
            id: None,
            needs_update: false,
            image: image
        })))
    }
}


impl<F, T> TextureBuffer<F, T> {
    /// Creates a new `TextureBuffer`.
    pub fn new(factory: F) -> TextureBuffer<F, T> {
        TextureBuffer {
            factory: factory,
            textures: HashMap::new(),
            next_id: 0,
        }
    }
}

impl Texture {
    /// Edit image.
    pub fn with_image_mut<F>(&self, f: F)
        where F: FnOnce(&mut RgbaImage) {
        let mut inner = self.0.write().unwrap();
        f(&mut inner.image);
        inner.needs_update = true;
    }
}
