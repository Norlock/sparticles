use super::{Camera, Clock, EmitterState, Events, GfxState, Material, MaterialRef, Mesh, MeshRef};
use crate::init::{InitEmitters, InitSettings};
use crate::loader::{Model, BUILTIN_ID};
use crate::traits::*;
use crate::ui::GuiState;
use crate::util::ID;
use crate::{fx::PostProcessState, AppSettings};
use egui_winit::winit::{dpi::PhysicalSize, event::KeyboardInput, window::Window};
use std::collections::HashMap;

pub struct State {
    pub camera: Camera,
    pub clock: Clock,
    pub emitters: Vec<EmitterState>,
    pub gui: GuiState,
    pub post_process: PostProcessState,
    pub gfx_state: GfxState,
    pub registered_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
    pub registered_em_anims: Vec<Box<dyn RegisterEmitterAnimation>>,
    pub registered_post_fx: Vec<Box<dyn RegisterPostFx>>,
    pub events: Events,
    pub collection: HashMap<ID, Model>,
}

pub trait FastFetch {
    fn get_mesh(&self, mesh_ref: &MeshRef) -> &Mesh;
    fn get_mat(&self, mat_ref: &MaterialRef) -> &Material;
}

impl FastFetch for HashMap<ID, Model> {
    fn get_mesh(&self, mesh_ref: &MeshRef) -> &Mesh {
        self.get(&mesh_ref.collection_id)
            .expect(&format!(
                "Collection doesn't exist: {:?}",
                &mesh_ref.collection_id
            ))
            .meshes
            .get(&mesh_ref.mesh_id)
            .expect(&format!("Mesh doesn't exist: {:?}", &mesh_ref.mesh_id))
    }

    fn get_mat(&self, mat_ref: &MaterialRef) -> &Material {
        self.get(&mat_ref.collection_id)
            .expect(&format!(
                "Collection doesn't exist: {:?}",
                &mat_ref.collection_id
            ))
            .materials
            .get(&mat_ref.material_id)
            .expect(&format!("Mesh doesn't exist: {:?}", &mat_ref.material_id))
    }
}

impl State {
    pub fn update(&mut self) {
        self.clock.update(&self.events);

        Camera::update(self);
        PostProcessState::update(self);
        EmitterState::update(self);
    }

    pub fn render(&mut self) {
        GfxState::render(self);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gfx_state.resize(size);
        self.post_process.resize(&self.gfx_state);
        self.camera.resize(&self.gfx_state);
    }

    pub fn process_events(&mut self, input: KeyboardInput, shift_pressed: bool) {
        if self.camera.process_input(&input) {
            return;
        }

        if GuiState::process_input(self, &input, shift_pressed) {
            return;
        }
    }

    pub fn new(app_settings: impl AppSettings, window: Window) -> Self {
        let mut gfx_state = pollster::block_on(GfxState::new(window));

        let clock = Clock::new();
        let camera = Camera::new(&gfx_state);
        let mut collection = HashMap::new();

        let builtin = Model::load_builtin(&gfx_state);
        collection.insert(builtin.id.to_string(), builtin);

        for em in app_settings.emitters().iter() {
            let mesh_key = &em.mesh.collection_id;
            let mat_key = &em.material.collection_id;

            if (mesh_key != &BUILTIN_ID) && !collection.contains_key(mesh_key) {
                collection.insert(
                    mesh_key.to_string(),
                    Model::load_gltf(&gfx_state, mesh_key).expect("Can't load model"),
                );
            }

            if (mat_key != &BUILTIN_ID) && !collection.contains_key(mat_key) {
                collection.insert(
                    mat_key.to_string(),
                    Model::load_gltf(&gfx_state, mat_key).expect("Can't load model"),
                );
            }
        }

        let InitEmitters {
            emitters,
            registered_em_anims,
            registered_par_anims,
        } = InitSettings::create_emitters(&app_settings, &gfx_state, &camera, &collection);

        let mut post_process = PostProcessState::new(&gfx_state, &app_settings);
        let registered_post_fx =
            InitSettings::create_post_fx(&app_settings, &gfx_state, &mut post_process);

        let gui = GuiState::new(app_settings.show_gui(), &mut gfx_state);

        Self {
            clock,
            camera,
            emitters,
            gui,
            post_process,
            gfx_state,
            registered_par_anims,
            registered_em_anims,
            registered_post_fx,
            events: Events::default(),
            collection,
        }
    }
}
