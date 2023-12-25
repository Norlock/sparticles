use super::{
    Camera, Clock, EmitterState, GfxState, Material, MaterialRef, Mesh, MeshRef, SparEvents,
};
use crate::fx::PostProcessState;
use crate::init::{AppVisitor, Init};
use crate::loader::Model;
use crate::terrain::TerrainGenerator;
use crate::traits::*;
use crate::util::ID;
use async_std::sync::RwLock;
use async_std::task;
use egui_winit::winit::{dpi::PhysicalSize, event::KeyboardInput, window::Window};
use std::collections::HashMap;
use std::sync::Arc;

/// Sparticles state
pub struct SparState {
    pub camera: Camera,
    pub clock: Clock,
    pub emitters: Vec<EmitterState>,
    pub post_process: PostProcessState,
    pub gfx: Arc<RwLock<GfxState>>,
    pub collection: Arc<RwLock<HashMap<ID, Model>>>,
    pub play: bool,
    pub registry_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
    pub registry_em_anims: Vec<Box<dyn RegisterEmitterAnimation>>,
    pub registered_post_fx: Vec<Box<dyn RegisterPostFx>>,
    pub terrain_generator: TerrainGenerator,
}

pub trait FastFetch {
    fn get_mesh(&self, mesh_ref: &MeshRef) -> &Mesh;
    fn get_mat(&self, mat_ref: &MaterialRef) -> &Material;
}

impl FastFetch for HashMap<ID, Model> {
    fn get_mesh(&self, mesh_ref: &MeshRef) -> &Mesh {
        self.get(&mesh_ref.collection_id)
            .unwrap_or_else(|| panic!("Collection doesn't exist: {:?}", &mesh_ref.collection_id))
            .meshes
            .get(&mesh_ref.mesh_id)
            .unwrap_or_else(|| panic!("Mesh doesn't exist: {:?}", &mesh_ref.mesh_id))
    }

    fn get_mat(&self, mat_ref: &MaterialRef) -> &Material {
        self.get(&mat_ref.collection_id)
            .unwrap_or_else(|| panic!("Collection doesn't exist: {:?}", &mat_ref.collection_id))
            .materials
            .get(&mat_ref.material_id)
            .unwrap_or_else(|| panic!("Mesh doesn't exist: {:?}", &mat_ref.material_id))
    }
}

impl SparState {
    pub async fn update(&mut self, events: &SparEvents) {
        self.clock.update(self.play);

        if events.toggle_play {
            self.play = !self.play;
        }

        Camera::update(self, events).await;
        PostProcessState::update(self, events).await;
        EmitterState::update(self, events).await;
        TerrainGenerator::update(self).await;
    }

    pub async fn resize(&mut self, size: PhysicalSize<u32>) {
        let mut gfx = self.gfx.write().await;
        gfx.resize(size);
        self.post_process.resize(&gfx);
        self.camera.resize(&gfx);
    }

    pub fn process_events(&mut self, input: &KeyboardInput) {
        self.camera.process_input(input);
    }

    pub fn egui_ctx(&self) -> egui_winit::egui::Context {
        let gfx = task::block_on(self.gfx.read());
        gfx.ctx.clone()
    }

    pub async fn new(init: &mut impl AppVisitor, window: Window) -> Self {
        let gfx = GfxState::new(window).await;
        let clock = Clock::default();
        let camera = Camera::new(&gfx);
        let builtin = Model::load_builtin(&gfx);

        let mut collection = HashMap::new();

        collection.insert(builtin.id.to_string(), builtin);

        let mut post_process = PostProcessState::new(&gfx, init);
        let terrain_generator = TerrainGenerator::new(&gfx, &camera);
        let gfx = Arc::new(RwLock::new(gfx));
        let collection = Arc::new(RwLock::new(collection));

        let init_settings = Init::new(
            init,
            &gfx,
            &camera,
            &collection,
            &mut post_process,
            &terrain_generator.cube_bg_layout,
        )
        .await;

        let mut state = Self {
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
            terrain_generator,
        };

        init.add_widget_builders(&mut state);

        state
    }
}

//#[cfg(test)]
//mod tests {
//use std::sync::{Arc, Mutex};

//pub struct Jep {
//a: String,
//}

//pub struct Test {
//a: String,
//b: Arc<Mutex<Vec<Jep>>>,
//}

//#[test]
//fn it_works() {
//let mut jeppers = vec![];
//for _i in 0..10 {
//jeppers.push(Jep {
//a: "jep".to_owned(),
//})
//}

//let mut b = Test {
//a: "nep".to_owned(),
//b: Arc::new(Mutex::new(jeppers)),
//};

//let ref mut a = b.b.lock().unwrap()[0];

//test(&mut b, a);

//let result = 2 + 2;
//assert_eq!(result, 4);
//}

//fn test(a: &mut Test, jep: &mut Jep) {
//assert_ne!(a.a, jep.a);
//}
//}
