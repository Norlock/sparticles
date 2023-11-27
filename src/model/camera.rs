use super::{gfx_state::GfxState, State};
use egui_wgpu::wgpu;
use egui_winit::{
    egui::WidgetText,
    winit::event::{ElementState, KeyboardInput, VirtualKeyCode},
};
use encase::UniformBuffer;
use glam::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4 {
    x_axis: Vec4::new(1.0, 0.0, 0.0, 0.0),
    y_axis: Vec4::new(0.0, 1.0, 0.0, 0.0),
    z_axis: Vec4::new(0.0, 0.0, 0.5, 0.5),
    w_axis: Vec4::new(0.0, 0.0, 0.0, 1.0),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TonemapType {
    AcesNarkowicz,
    AcesHill,
    Uchimura,
    Lottes,
}

impl From<TonemapType> for WidgetText {
    fn from(value: TonemapType) -> Self {
        match value {
            TonemapType::AcesNarkowicz => "ACES Narkowicz".into(),
            TonemapType::AcesHill => "ACES Hill".into(),
            TonemapType::Uchimura => "Uchimura".into(),
            TonemapType::Lottes => "Lottes".into(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Camera {
    pub position: Vec3, // Camera position
    pub view_dir: Vec3, // Camera aimed at
    pub pitch: f32,
    pub yaw: f32,
    pub bg_layout: wgpu::BindGroupLayout,
    pub bloom_treshold: Vec3, // To prepare for post FX
    pub tonemap_type: TonemapType,
    pub look_at: Vec3,
    fov: f32,  // Field of view (frustum vertical degrees)
    near: f32, // What is too close to show
    far: f32,  // What is too far to show
    buffer: wgpu::Buffer,

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

    proj: Mat4,
    bg: wgpu::BindGroup,
}

#[derive(encase::ShaderType)]
struct CameraUniform {
    view_proj: glam::Mat4,
    view: glam::Mat4,
    position: glam::Vec3,
    bloom_treshold: glam::Vec3,
    tonemap: u32,
}

impl Camera {
    pub fn bg(&self) -> &wgpu::BindGroup {
        &self.bg
    }

    pub fn new(gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;

        let position = Vec3::new(0., 0., 10.);
        let view_dir = Vec3::new(0., 0., -10.);
        let look_at = position + view_dir;
        let pitch = 0.;
        let yaw = 0.;
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

        let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bg_layout,
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
            position,
            view_dir,
            look_at,
            buffer,
            bg_layout,
            bg,
            bloom_treshold: Vec3::MAX,
            tonemap_type: TonemapType::AcesNarkowicz,
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
            events,
            ..
        } = state;

        if events.reset_camera().is_some() {
            camera.pitch = 0.;
            camera.yaw = 0.;
            camera.position = glam::Vec3::new(0., 0., 10.);
            camera.view_dir = glam::Vec3::new(0., 0., -10.);
        }

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

        let buf_content = camera.create_buffer_content();
        queue.write_buffer(&camera.buffer, 0, &buf_content);
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

    fn create_buffer_content(&mut self) -> Vec<u8> {
        let view_mat = self.view_mat();
        let view_proj = self.view_proj(&view_mat);

        let uniform = CameraUniform {
            view_proj,
            view: view_mat,
            position: self.position,
            bloom_treshold: self.bloom_treshold,
            tonemap: self.tonemap_type as u32,
        };

        let mut buffer = UniformBuffer::new(vec![]);
        buffer.write(&uniform).unwrap();
        buffer.into_inner()
    }

    pub fn view_mat(&self) -> Mat4 {
        let yaw_mat = Mat3::from_rotation_y(self.yaw);
        let pitch_mat = Mat3::from_rotation_x(self.pitch);

        let rotated_view_dir = yaw_mat * pitch_mat * self.view_dir;
        Mat4::look_at_rh(self.position, self.position + rotated_view_dir, Vec3::Y)
    }

    pub fn view_proj(&self, view_mat: &Mat4) -> Mat4 {
        OPENGL_TO_WGPU_MATRIX * self.proj * (*view_mat)
    }
}

fn buffer_size() -> u64 {
    let view_proj_size = 16;
    let view_mat_size = 16;
    let position_size = 4;
    let bloom_treshold_size = 4;
    let tonemap_size = 4;

    // The most aligned member of that strut is aligned to 16. As such
    // destruct is aligned to 16, instructs have their size rounded up to their alignment.
    // So bloom treshold 1 == 4

    (view_proj_size + view_mat_size + position_size + bloom_treshold_size + tonemap_size)
        * std::mem::size_of::<f32>() as u64
}
