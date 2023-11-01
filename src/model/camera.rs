use super::{gfx_state::GfxState, State};
use crate::traits::ToVecF32;
use egui_wgpu::wgpu;
use egui_winit::winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use glam::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4 {
    x_axis: Vec4::new(1.0, 0.0, 0.0, 0.0),
    y_axis: Vec4::new(0.0, 1.0, 0.0, 0.0),
    z_axis: Vec4::new(0.0, 0.0, 0.5, 0.5),
    w_axis: Vec4::new(0.0, 0.0, 0.0, 1.0),
};

type Mat4x2 = [[f32; 2]; 4];

#[allow(dead_code)]
#[derive(Debug)]
pub struct Camera {
    pub position: Vec3, // Camera position
    pub view_dir: Vec3, // Camera aimed at
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    look_at: Vec3,
    fov: f32,  // Field of view (frustum vertical degrees)
    near: f32, // What is too close to show
    far: f32,  // What is too far to show
    buffer: wgpu::Buffer,

    // TODO last_key_pressed
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_rotate_left_pressed: bool,
    is_right_pressed: bool,
    is_rotate_right_pressed: bool,
    is_up_pressed: bool,
    is_rotate_up_pressed: bool,
    is_down_pressed: bool,
    is_rotate_down_pressed: bool,

    vertex_positions: Mat4x2,
    pub proj: Mat4,
}

impl Camera {
    pub fn new(gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;

        let position = Vec3::new(0., 0., 10.);
        let view_dir = Vec3::new(0., 0., -10.);
        let look_at = position + view_dir;
        let vertex_positions = vertex_positions();
        let pitch = 0.;
        let yaw = 0.;
        let roll = 0.;
        let near = 0.1;
        let far = 100.0;
        let fov = (45.0f32).to_radians();
        let aspect = gfx_state.aspect();
        let proj = Mat4::perspective_rh(fov, aspect, near, far);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size(),
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
            fov,
            far,
            near,
            pitch,
            yaw,
            roll,
            position,
            view_dir,
            look_at,
            buffer,
            bind_group_layout,
            bind_group,
            vertex_positions,
            proj,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_rotate_left_pressed: false,
            is_right_pressed: false,
            is_rotate_right_pressed: false,
            is_up_pressed: false,
            is_rotate_up_pressed: false,
            is_down_pressed: false,
            is_rotate_down_pressed: false,
        }
    }

    pub fn update(state: &mut State) {
        let State {
            gfx_state,
            camera,
            clock,
            ..
        } = state;

        let queue = &gfx_state.queue;
        let speed = 3.0;

        let move_delta = speed * clock.delta_sec();
        let rotation = move_delta / 3.0;
        let yaw_mat = Mat3::from_rotation_y(camera.yaw);
        let pitch_mat = Mat3::from_rotation_x(camera.pitch);

        let rotate_vec = |unrotated_vec: Vec3| yaw_mat * pitch_mat * unrotated_vec;

        if camera.is_forward_pressed {
            camera.position += rotate_vec(Vec3::new(0., 0., -move_delta));
        }

        if camera.is_backward_pressed {
            camera.position += rotate_vec(Vec3::new(0., 0., move_delta));
        }

        if camera.is_up_pressed {
            camera.position.y += move_delta;
        }

        if camera.is_down_pressed {
            camera.position.y -= move_delta;
        }

        if camera.is_left_pressed {
            camera.position += rotate_vec(Vec3::new(-move_delta, 0., 0.));
        }

        if camera.is_right_pressed {
            camera.position += rotate_vec(Vec3::new(move_delta, 0., 0.));
        }

        if camera.is_rotate_up_pressed {
            camera.pitch += rotation;
        }

        if camera.is_rotate_down_pressed {
            camera.pitch -= rotation;
        }

        if camera.is_rotate_left_pressed {
            camera.yaw += rotation;
        }

        if camera.is_rotate_right_pressed {
            camera.yaw -= rotation;
        }

        let buf_content_raw = camera.create_buffer_content();
        let buf_content = bytemuck::cast_slice(&buf_content_raw);

        queue.write_buffer(&camera.buffer, 0, buf_content);
    }

    pub fn resize(&mut self, gfx_state: &GfxState) {
        self.proj = Mat4::perspective_rh(self.fov, gfx_state.aspect(), self.near, self.far);
    }

    pub fn process_input(&mut self, input: &KeyboardInput) -> bool {
        let press_state = input.state;
        let keycode = input.virtual_keycode.unwrap_or(VirtualKeyCode::Return);
        let is_pressed = press_state == ElementState::Pressed;

        match keycode {
            VirtualKeyCode::W => {
                self.is_forward_pressed = is_pressed;
            }
            VirtualKeyCode::A => {
                self.is_left_pressed = is_pressed;
            }
            VirtualKeyCode::S => {
                self.is_backward_pressed = is_pressed;
            }
            VirtualKeyCode::D => {
                self.is_right_pressed = is_pressed;
            }
            VirtualKeyCode::Up => {
                self.is_rotate_up_pressed = is_pressed;
            }
            VirtualKeyCode::Left => {
                self.is_rotate_left_pressed = is_pressed;
            }
            VirtualKeyCode::Right => {
                self.is_rotate_right_pressed = is_pressed;
            }
            VirtualKeyCode::Down => {
                self.is_rotate_down_pressed = is_pressed;
            }
            VirtualKeyCode::LControl => {
                self.is_down_pressed = is_pressed;
            }
            VirtualKeyCode::Space => {
                self.is_up_pressed = is_pressed;
            }
            _ => return false,
        }

        true
    }

    fn create_buffer_content(&mut self) -> Vec<f32> {
        let yaw_mat = Mat3::from_rotation_y(self.yaw);
        let pitch_mat = Mat3::from_rotation_x(self.pitch);

        let rotated_view_dir = yaw_mat * pitch_mat * self.view_dir;
        let view_mat = Mat4::look_at_rh(self.position, self.position + rotated_view_dir, Vec3::Y);

        let view_proj = OPENGL_TO_WGPU_MATRIX * self.proj * view_mat;

        let view_proj_arr = view_proj.to_cols_array().to_vec();
        let view_arr = view_mat.to_cols_array().to_vec();
        let rotated_vertices_arr = self.get_rotated_vertices(view_proj);
        let vertex_positions_arr = self.vertex_positions.to_vec_f32();
        let view_pos_arr = self.position.to_vec_f32();

        [
            view_proj_arr,
            view_arr,
            rotated_vertices_arr,
            vertex_positions_arr,
            view_pos_arr,
        ]
        .concat()
    }

    fn get_rotated_vertices(&self, view_proj: Mat4) -> Vec<f32> {
        let camera_right = view_proj.row(0).truncate().normalize();
        let camera_up = view_proj.row(1).truncate().normalize();

        self.vertex_positions
            .into_iter()
            .map(|v_pos| camera_right * v_pos[0] + camera_up * v_pos[1])
            .flat_map(|v3| vec![v3.x, v3.y, v3.z, 1.])
            .collect::<Vec<f32>>()
    }
}

impl ToVecF32 for Mat4x2 {
    fn to_vec_f32(&self) -> Vec<f32> {
        self.iter().flatten().copied().collect()
    }
}

impl ToVecF32 for Vec3 {
    fn to_vec_f32(&self) -> Vec<f32> {
        vec![self.x, self.y, self.z, 0.0]
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

fn buffer_size() -> u64 {
    let view_proj_size = 16;
    let view_mat_size = 16;
    let rotated_vertices_size = 16;
    let vertex_positions_size = 12;
    let view_pos_size = 4;
    let f32_mem_size = 4;

    (view_proj_size + view_mat_size + rotated_vertices_size + vertex_positions_size + view_pos_size)
        * f32_mem_size
}
