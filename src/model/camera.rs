use egui_wgpu_backend::wgpu::{self};
use glam::*;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use super::{
    gfx_state::{self, GfxState},
    Clock,
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4 {
    x_axis: Vec4::new(1.0, 0.0, 0.0, 0.0),
    y_axis: Vec4::new(0.0, 1.0, 0.0, 0.0),
    z_axis: Vec4::new(0.0, 0.0, 0.5, 0.5),
    w_axis: Vec4::new(0.0, 0.0, 0.0, 1.0),
};

type Mat4x2 = [[f32; 2]; 4];

#[allow(dead_code)]
pub struct Camera {
    look_from: glam::Vec3, // Camera position
    look_at: glam::Vec3,   // Camera aimed at
    up: glam::Vec3,        // What way is up
    fov: f32,              // Field of view (frustum vertical degrees)
    aspect: f32,           // Make sure x/y stays in aspect
    near: f32,             // What is too close to show
    far: f32,              // What is too far to show
    buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,

    vertex_positions: Mat4x2,
    forward_length: f32,
    proj: Mat4,
}

impl Camera {
    pub fn new(gfx_state: &gfx_state::GfxState) -> Self {
        let device = &gfx_state.device;
        let surface_config = &gfx_state.surface_config;

        let look_from = glam::Vec3::new(0., 0., 10.);
        let look_at = glam::Vec3::new(0., 0., 0.);
        let vertex_positions = vertex_positions();
        let forward_length = (look_from - look_at).length();

        let near = 0.1;
        let far = 100.0;
        let fov = (45.0f32).to_radians();
        let up = Vec3::Y;
        let speed = 0.01;

        let aspect = create_aspect(surface_config);
        let proj = Mat4::perspective_rh(fov, aspect, near, far);

        let view_proj_size = 16;
        let view_mat_size = 16;
        let rotated_vertices_size = 16;
        let vertex_positions_size = 12;
        let view_pos_size = 4;
        let f32_mem_size = 4;

        let buff_size = (view_proj_size
            + view_mat_size
            + rotated_vertices_size
            + vertex_positions_size
            + view_pos_size)
            * f32_mem_size;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: buff_size, // F32 fields * 4
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label: Some("Camera buffer"),
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            up,
            fov,
            far,
            near,
            look_from,
            buffer,
            bind_group_layout,
            bind_group,
            look_at,
            aspect,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            speed,
            vertex_positions,
            forward_length,
            proj,
        }
    }

    fn recalculate_forward(&mut self) {
        self.forward_length = (self.look_from - self.look_at).length();
    }

    pub fn update(&mut self, gfx_state: &GfxState, clock: &Clock) {
        let queue = &gfx_state.queue;
        let forward = self.look_from - self.look_at;
        let forward_norm = forward.normalize();
        let rotation = 1f32 * clock.delta_sec();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed {
            self.look_from -= forward_norm * self.speed;
            self.recalculate_forward();
        }

        if self.is_backward_pressed {
            self.look_from += forward_norm * self.speed;
            self.recalculate_forward();
        }

        let mut rotate = |theta: f32| {
            let forward = self.look_from - self.look_at;
            let sin_cos = Vec2::new(theta.cos(), theta.sin());
            let rotation = Mat2::from_cols_array(&[sin_cos.x, -sin_cos.y, sin_cos.y, sin_cos.x]);

            let Vec2 { x, y: z } = rotation * forward.xz();

            let new_forward =
                Vec3::new(x, forward.y, z).clamp_length(self.forward_length, self.forward_length);

            self.look_from = self.look_at + new_forward;
        };

        if self.is_left_pressed {
            rotate(-rotation);
        }

        if self.is_right_pressed {
            rotate(rotation);
        }

        let buf_content_raw = self.create_buffer_content();
        let buf_content = bytemuck::cast_slice(&buf_content_raw);

        queue.write_buffer(&self.buffer, 0, buf_content);
    }

    pub fn window_resize(&mut self, gfx_state: &gfx_state::GfxState) {
        self.aspect = create_aspect(&gfx_state.surface_config);
        self.proj = Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far);
    }

    pub fn process_input(&mut self, input: KeyboardInput) {
        let state = input.state;
        let keycode = input.virtual_keycode.unwrap_or(VirtualKeyCode::Return);
        let is_pressed = state == ElementState::Pressed;

        match keycode {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.is_forward_pressed = is_pressed;
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.is_left_pressed = is_pressed;
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.is_backward_pressed = is_pressed;
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.is_right_pressed = is_pressed;
            }
            VirtualKeyCode::Minus => {
                self.is_backward_pressed = is_pressed;
            }
            _ => (),
        }
    }

    fn create_buffer_content(&self) -> Vec<f32> {
        let view = Mat4::look_at_rh(self.look_from, self.look_at, self.up);

        let view_proj = OPENGL_TO_WGPU_MATRIX * self.proj * view;
        let view_proj_arr = view_proj.to_cols_array().to_vec();
        let view_arr = view.to_cols_array().to_vec();
        let rotated_vertices_arr = self.get_rotated_vertices(view_proj);
        let vertex_positions: Vec<f32> = self.vertex_positions.into_iter().flatten().collect();
        let view_pos = self.look_from.extend(0.0).to_array().to_vec();

        [
            view_proj_arr,
            view_arr,
            rotated_vertices_arr,
            vertex_positions,
            view_pos,
        ]
        .concat()
    }

    fn get_rotated_vertices(&self, view_proj: Mat4) -> Vec<f32> {
        let camera_right = view_proj.row(0).truncate().normalize();
        let camera_up = view_proj.row(1).truncate().normalize();

        self.vertex_positions
            .into_iter()
            .map(|v_pos| camera_right * v_pos[0] + camera_up * v_pos[1])
            .map(|v3| vec![v3.x, v3.y, v3.z, 1.0])
            .flatten()
            .collect::<Vec<f32>>()
    }
}

fn vertex_positions() -> Mat4x2 {
    [
        Vec2::new(-1., -1.).into(),
        Vec2::new(1., -1.).into(),
        Vec2::new(-1., 1.).into(),
        Vec2::new(1., 1.).into(),
    ]
}

fn create_aspect(surface_config: &wgpu::SurfaceConfiguration) -> f32 {
    surface_config.width as f32 / surface_config.height as f32
}
