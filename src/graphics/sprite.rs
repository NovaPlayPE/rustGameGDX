use std::borrow::Borrow;
use std::rc::Rc;
use std::thread;

use glium::{DrawError, GlObject, Surface, uniform};
use glium::uniforms::{Sampler, SamplerBehavior};
pub use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerWrapFunction};
use maybe_owned::MaybeOwned;

use crate::graphics::texture::{TextureRegion, TextureRegionHolder};

const VERTEX_SHADER_SRC: &str = include_str!("shaders/sprite.vs.glsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/sprite.fs.glsl");

const QUAD_VERTEX_SIZE: usize = 4;
const QUAD_INDEX_SIZE: usize = 6;
const BATCH_SIZE: usize = 1024;
const BATCH_VERTEX_SIZE: usize = QUAD_VERTEX_SIZE * BATCH_SIZE;
const BATCH_INDEX_SIZE: usize = QUAD_INDEX_SIZE * BATCH_SIZE;


#[derive(Clone, Copy, Debug)]
pub struct VertexData {
    pos: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}
glium::implement_vertex!(VertexData, pos, tex_coords, color);

#[derive(Clone, Copy, Debug, Default)]
pub struct SpriteDrawParams {
    pub sampler_behavior: SamplerBehavior,
    pub alpha_blending: bool,
}

impl SpriteDrawParams {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn alpha(mut self, alpha: bool) -> Self {
        self.alpha_blending = alpha;
        self
    }

    pub fn wrap_function(mut self, function: SamplerWrapFunction) -> Self {
        self.sampler_behavior.wrap_function = (function, function, function);
        self
    }

    pub fn minify_filter(mut self, filter: MinifySamplerFilter) -> Self {
        self.sampler_behavior.minify_filter = filter;
        self
    }

    pub fn magnify_filter(mut self, filter: MagnifySamplerFilter) -> Self {
        self.sampler_behavior.magnify_filter = filter;
        self
    }
}

pub struct SpriteBatch<'a, 'b, S>
    where S: 'b + Surface
{
    renderer: &'a mut SpriteRenderer,
    target: &'b mut S,
    draw_params: SpriteDrawParams,
    draw_calls: u32,
    finished: bool,
}

impl<'a, 'b, S> SpriteBatch<'a, 'b, S>
    where S: Surface
{
    fn new(renderer: &'a mut SpriteRenderer, draw_params: SpriteDrawParams, target: &'b mut S) -> Self {
        renderer.sprite_queue.clear();

        SpriteBatch {
            renderer,
            target,
            draw_params,
            draw_calls: 0,
            finished: false,
        }
    }

    pub fn draw(&mut self, sprite: &Sprite) -> Result<(), DrawError> {
        if self.renderer.sprite_queue.len() == BATCH_SIZE {
            self.flush()?;
        }

        let vertices = sprite.get_vertex_data();
        self.renderer.sprite_queue.push(vertices, sprite.rc_texture().clone());

        Ok(())
    }

    pub fn finish(mut self) -> Result<u32, DrawError> {
        self.flush()?;
        self.finished = true;
        Ok(self.draw_calls)
    }

    fn flush(&mut self) -> Result<(), DrawError> {
        if self.renderer.sprite_queue.vertices.is_empty() {
            return Ok(());
        }

        let params = {
            let blend = if self.draw_params.alpha_blending {
                glium::Blend::alpha_blending()
            } else {
                Default::default()
            };
            glium::DrawParameters {
                blend,
                .. Default::default()
            }
        };

        {
            let vertex_buffer = self.renderer.vertex_buffer.slice(0..self.renderer.sprite_queue.vertices.len())
                .expect("Vertex buffer does not contain enough elements!");
            vertex_buffer.write(&self.renderer.sprite_queue.vertices);
        }

        let mut render_texture = self.renderer.sprite_queue.textures[0].clone();
        let mut offset = 0;
        for (i, texture) in self.renderer.sprite_queue.textures.iter().enumerate().skip(1) {
            if texture.get_id() != render_texture.get_id() {
                {
                    let sampler: Sampler<glium::Texture2d> = glium::uniforms::Sampler(
                        render_texture.borrow(),
                        self.draw_params.sampler_behavior,
                    );
                    let uniforms = uniform! {
                        image: sampler,
                        projectionView: *self.renderer.projection_matrix.as_ref(),
                    };

                    let (vertex_start, vertex_end) = (offset * QUAD_VERTEX_SIZE, i * QUAD_VERTEX_SIZE);
                    let vertex_buffer = self.renderer.vertex_buffer.slice(vertex_start..vertex_end)
                        .expect("Vertex buffer does not contain enough elements!");
                    let (index_start, index_end) = (offset * QUAD_INDEX_SIZE, i * QUAD_INDEX_SIZE);
                    let index_buffer = self.renderer.index_buffer.slice(index_start..index_end)
                        .expect("Index buffer does not contain enough elements!");

                    self.target.draw(vertex_buffer, index_buffer, &self.renderer.shader, &uniforms, &params)?;
                }

                self.draw_calls += 1;

                offset = i;
                render_texture = texture.clone();
            }
        }

        {
            let i = self.renderer.sprite_queue.len();

            let sampler: Sampler<glium::Texture2d> = glium::uniforms::Sampler(
                render_texture.borrow(),
                self.draw_params.sampler_behavior,
            );
            let uniforms = uniform! {
                image: sampler,
                projectionView: *self.renderer.projection_matrix.as_ref(),
            };

            let (vertex_start, vertex_end) = (offset * QUAD_VERTEX_SIZE, i * QUAD_VERTEX_SIZE);
            let vertex_buffer = self.renderer.vertex_buffer.slice(vertex_start..vertex_end)
                .expect("Vertex buffer does not contain enough elements!");
            let (index_start, index_end) = (offset * QUAD_INDEX_SIZE, i * QUAD_INDEX_SIZE);
            let index_buffer = self.renderer.index_buffer.slice(index_start..index_end)
                .expect("Index buffer does not contain enough elements!");

            self.target.draw(vertex_buffer, index_buffer, &self.renderer.shader, &uniforms, &params)?;

            self.draw_calls += 1;
        }

        self.renderer.sprite_queue.clear();

        Ok(())
    }
}

impl<'a, 'b, S> Drop for SpriteBatch<'a, 'b, S>
    where S: Surface
{
    #[inline]
    fn drop(&mut self) {
        if !thread::panicking() {
            assert!(self.finished, "The `SpriteBatch` object must be explicitly destroyed \
                                    by calling `.finish()`");
        }
    }
}

#[derive(Debug)]
pub struct SpriteQueue {
    vertices: Vec<VertexData>,
    textures: Vec<Rc<glium::Texture2d>>,
}

impl SpriteQueue {
    fn new() -> Self {
        SpriteQueue {
            vertices: Vec::with_capacity(BATCH_VERTEX_SIZE),
            textures: Vec::with_capacity(BATCH_SIZE),
        }
    }

    fn push(&mut self, vertices: [VertexData; 4], texture: Rc<glium::Texture2d>) {
        assert!(self.textures.len() < BATCH_SIZE, "Sprite queue is full!");

        self.vertices.extend_from_slice(&vertices);
        self.textures.push(texture);
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.textures.clear();
    }

    fn len(&self) -> usize {
        self.textures.len()
    }
}

#[derive(Debug)]
pub struct SpriteRenderer {
    projection_matrix: glm::Mat4,
    shader: glium::Program,
    vertex_buffer: glium::VertexBuffer<VertexData>,
    index_buffer: glium::IndexBuffer<u16>,
    sprite_queue: SpriteQueue,
}

impl SpriteRenderer {
    pub fn new<F: glium::backend::Facade>(display: &F, projection: glm::Mat4) -> Self {
        let program_creation_input = glium::program::ProgramCreationInput::SourceCode {
            vertex_shader: VERTEX_SHADER_SRC,
            fragment_shader: FRAGMENT_SHADER_SRC,
            geometry_shader: None,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            transform_feedback_varyings: None,
            outputs_srgb: true,
            uses_point_size: false,
        };
        let shader = glium::Program::new(display, program_creation_input)
            .expect("Could not create SpriteRenderer shader program.");

        Self::with_shader(display, shader, projection)
    }

    pub fn with_shader<F: glium::backend::Facade>(display: &F, shader: glium::Program,
                                                  projection: glm::Mat4) -> Self {
        let vertex_buffer = glium::VertexBuffer::empty_dynamic(
            display,
            BATCH_VERTEX_SIZE,
        ).expect("Could not create SpriteRenderer vertex buffer.");

        let mut indices = Vec::with_capacity(BATCH_INDEX_SIZE);
        for quad_index in 0..BATCH_SIZE {
            let offset = quad_index as u16 * QUAD_VERTEX_SIZE as u16;
            let new_indices = [
                0 + offset, 1 + offset, 2 + offset,
                0 + offset, 2 + offset, 3 + offset,
            ];
            indices.extend_from_slice(&new_indices);
        }
        let index_buffer = glium::IndexBuffer::immutable(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &indices,
        ).expect("Could not create SpriteRenderer index buffer.");

        Self {
            projection_matrix: projection,
            shader,
            vertex_buffer,
            index_buffer,
            sprite_queue: SpriteQueue::new(),
        }
    }

    pub fn begin_batch<'a, 'b, S: Surface>(&'a mut self, draw_params: SpriteDrawParams, target: &'b mut S) -> SpriteBatch<'a, 'b, S> {
        SpriteBatch::new(self, draw_params, target)
    }

    pub fn draw<S: Surface>(&self, sprite: &Sprite, draw_params: SpriteDrawParams, target: &mut S) {
        let vertices = sprite.get_vertex_data();

        let vertex_buffer = self.vertex_buffer.slice(0..QUAD_VERTEX_SIZE)
            .expect("Vertex buffer does not contain enough elements!");
        vertex_buffer.write(&vertices);

        let sampler: Sampler<glium::Texture2d> = glium::uniforms::Sampler(
            sprite.texture(),
            draw_params.sampler_behavior,
        );

        let uniforms = uniform! {
            image: sampler,
            projectionView: *self.projection_matrix.as_ref(),
        };

        let blend = if draw_params.alpha_blending {
            glium::Blend::alpha_blending()
        } else {
            Default::default()
        };
        let params = glium::DrawParameters {
            blend,
            .. Default::default()
        };

        let index_buffer = self.index_buffer.slice(0..QUAD_INDEX_SIZE)
            .expect("Index buffer does not contain enough elements!");

        target.draw(vertex_buffer, index_buffer, &self.shader, &uniforms, &params)
            .expect("Failed to draw sprites.");
    }

    pub fn set_projection_matrix(&mut self, projection: glm::Mat4) {
        self.projection_matrix = projection;
    }

    pub fn get_projection_matrix(&self) -> glm::Mat4 {
        self.projection_matrix
    }
}

#[derive(Clone)]
pub struct Sprite<'a> {
    texture_region: MaybeOwned<'a, TextureRegion>,

    position: glm::TVec2<f32>,
    origin: glm::TVec2<f32>,
    rotation: f32,
    scale: glm::TVec2<f32>,
    color: [f32; 4],
    flip_x: bool,
    flip_y: bool,
}

impl<'a> Sprite<'a> {
    pub fn new(texture: Rc<glium::Texture2d>) -> Self {
        let texture_region = TextureRegion::new(texture);
        Sprite::from_texture_region(texture_region)
    }

    pub fn with_sub_field(texture: Rc<glium::Texture2d>, offset: (u32, u32), size: (u32, u32)) -> Self {
        let texture_region = TextureRegion::with_sub_field(texture, offset, size);
        Sprite::from_texture_region(texture_region)
    }

    pub fn from_texture_region<T>(texture_region: T) -> Self
        where T: Into<MaybeOwned<'a, TextureRegion>>
    {
        Sprite {
            texture_region: texture_region.into(),

            position: glm::vec2(0.0, 0.0),
            origin: glm::vec2(0.5, 0.5),
            rotation: 0.0,
            scale: glm::vec2(1.0, 1.0),
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.position = glm::vec2(x, y);
        self
    }

    pub fn position(&self) -> (f32, f32) {
        (self.position.x, self.position.y)
    }

    pub fn set_origin(&mut self, x: f32, y: f32) {
        self.origin = glm::vec2(x, y);
    }

    pub fn origin(&self) -> (f32, f32) {
        (self.origin.x, self.origin.y)
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.scale = glm::vec2(scale_x, scale_y);
    }

    pub fn set_uniform_scale(&mut self, scale: f32) {
        self.set_scale(scale, scale);
    }

    pub fn scale(&self) -> (f32, f32) {
        (self.scale.x, self.scale.y)
    }

    pub fn set_flip_x(&mut self, flip_x: bool) {
        self.flip_x = flip_x;
    }

    pub fn flip_x(&self) -> bool {
        self.flip_x
    }

    pub fn set_flip_y(&mut self, flip_y: bool) {
        self.flip_y = flip_y;
    }

    pub fn flip_y(&self) -> bool {
        self.flip_y
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn color(&self) -> [f32; 4] {
        self.color
    }

    fn get_vertex_data(&self) -> [VertexData; 4] {
        let model = {
            let size = self.size();
            let scaled_size = glm::vec2(size.x as f32 * self.scale.x, size.y as f32 * self.scale.y);
            let pixel_origin = glm::vec2(scaled_size.x * self.origin.x, scaled_size.y * self.origin.y);
            let position = self.position - pixel_origin;
            let translate = glm::translation2d(&position);
            let rotate = if self.rotation != 0.0 {
                let rotation_matrix = glm::rotation2d(self.rotation.to_radians());
                glm::translation2d(&pixel_origin) * rotation_matrix * glm::translation2d(&-pixel_origin)
            } else {
                glm::identity()
            };
            let scale = glm::scaling2d(&scaled_size);
            translate * rotate * scale
        };

        let tex_coords = self.texture_coordinates();

        let tex_top_left = tex_coords[0];
        let tex_top_right = tex_coords[1];
        let tex_bottom_left = tex_coords[2];
        let tex_bottom_right = tex_coords[3];

        let (tex_top_left, tex_top_right, tex_bottom_left, tex_bottom_right) = match (self.flip_x(), self.flip_y()) {
            (false, false) => (tex_top_left, tex_top_right, tex_bottom_left, tex_bottom_right),
            (true, false) => (tex_top_right, tex_top_left, tex_bottom_right, tex_bottom_left),
            (false, true) => (tex_bottom_left, tex_bottom_right, tex_top_left, tex_top_right),
            (true, true) => (tex_bottom_right, tex_bottom_left, tex_top_right, tex_top_left),
        };

        let pos_top_left = model * glm::vec3(0.0, 1.0, 1.0);
        let pos_top_right = model * glm::vec3(1.0, 1.0, 1.0);
        let pos_bottom_left = model * glm::vec3(1.0, 0.0, 1.0);
        let pos_bottom_right = model * glm::vec3(0.0, 0.0, 1.0);

        let pos_top_left = [pos_top_left.x, pos_top_left.y];
        let pos_top_right = [pos_top_right.x, pos_top_right.y];
        let pos_bottom_left = [pos_bottom_left.x, pos_bottom_left.y];
        let pos_bottom_right = [pos_bottom_right.x, pos_bottom_right.y];

        let color = self.color();

        [
            VertexData { pos: pos_top_left, tex_coords: tex_top_left, color },
            VertexData { pos: pos_top_right, tex_coords: tex_top_right, color },
            VertexData { pos: pos_bottom_left, tex_coords: tex_bottom_right, color },
            VertexData { pos: pos_bottom_right, tex_coords: tex_bottom_left, color },
        ]
    }
}

impl<'a> TextureRegionHolder for Sprite<'a> {
    fn texture_region(&self) -> &TextureRegion {
        &self.texture_region
    }
}

pub trait DrawTexture {
    fn draw(&self, x: f32, y: f32) -> Sprite;
}

impl DrawTexture for TextureRegion {
    fn draw(&self, x: f32, y: f32) -> Sprite {
        let mut sprite = Sprite::from_texture_region(self);
        sprite.set_position(x, y);
        sprite
    }
}
