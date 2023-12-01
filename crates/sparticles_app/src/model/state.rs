use super::{
    Camera, Clock, EmitterState, GfxState, Material, MaterialRef, Mesh, MeshRef, SparEvents,
};
use crate::fx::PostProcessState;
use crate::init::{AppVisitor, Init};
use crate::loader::{Model, BUILTIN_ID};
use crate::traits::*;
use crate::util::ID;
use egui_winit::winit::{dpi::PhysicalSize, event::KeyboardInput, window::Window};
use std::collections::HashMap;

/// Sparticles state
pub struct SparState {
    pub camera: Camera,
    pub clock: Clock,
    pub emitters: Vec<EmitterState>,
    pub post_process: PostProcessState,
    pub gfx: GfxState,
    pub collection: HashMap<ID, Model>,
    pub play: bool,
    pub registry_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
    pub registry_em_anims: Vec<Box<dyn RegisterEmitterAnimation>>,
    pub registered_post_fx: Vec<Box<dyn RegisterPostFx>>,
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

impl SparState {
    pub fn update(&mut self, events: &SparEvents) {
        self.clock.update(self.play);

        if events.toggle_play {
            self.play = !self.play;
        }

        Camera::update(self, events);
        PostProcessState::update(self, events);
        EmitterState::update(self, events);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gfx.resize(size);
        self.post_process.resize(&self.gfx);
        self.camera.resize(&self.gfx);
    }

    pub fn process_events(&mut self, input: &KeyboardInput) {
        if self.camera.process_input(&input) {
            return;
        }
    }

    pub fn egui_ctx(&self) -> &egui_winit::egui::Context {
        &self.gfx.ctx
    }

    pub fn new(init: &mut impl AppVisitor, window: Window) -> Self {
        let mut gfx = pollster::block_on(GfxState::new(window));

        let clock = Clock::new();
        let camera = Camera::new(&gfx);
        let mut collection = HashMap::new();
        let mut post_process = PostProcessState::new(&gfx, init);

        let builtin = Model::load_builtin(&gfx);
        collection.insert(builtin.id.to_string(), builtin);

        let init_settings = Init::new(init, &gfx, &camera, &collection, &mut post_process);

        for em in init_settings.emitters.iter() {
            let uniform = &em.uniform;
            let mesh_key = &uniform.mesh.collection_id;
            let mat_key = &uniform.material.collection_id;

            if (mesh_key != &BUILTIN_ID) && !collection.contains_key(mesh_key) {
                collection.insert(
                    mesh_key.to_string(),
                    Model::load_gltf(&gfx, mesh_key).expect("Can't load model"),
                );
            }

            if (mat_key != &BUILTIN_ID) && !collection.contains_key(mat_key) {
                collection.insert(
                    mat_key.to_string(),
                    Model::load_gltf(&gfx, mat_key).expect("Can't load model"),
                );
            }
        }

        init.add_widget_builders(&mut gfx);

        Self {
            clock,
            camera,
            emitters: init_settings.emitters,
            post_process,
            gfx,
            registry_par_anims: init_settings.registry_par_anims,
            registry_em_anims: init_settings.registry_em_anims,
            registered_post_fx: init_settings.registry_post_fx,
            collection,
            play: true,
        }
    }
}