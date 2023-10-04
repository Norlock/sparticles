use egui_wgpu::wgpu;

use crate::model::spawn_state::SpawnOptions;
use crate::model::{Camera, Emitter, GfxState, SpawnState};
use crate::traits::*;

pub struct InitApp {
    pub show_gui: bool,
    pub light: SpawnInit,
    pub spawners: Vec<SpawnInit>,
}

pub struct SpawnInit {
    pub id: String,
    pub emitter: Emitter,
    pub particle_animations: Vec<Box<dyn CreateAnimation>>,
    pub emitter_animations: Vec<Box<dyn EmitterAnimation>>,
}

impl InitApp {
    pub fn create_light_spawner(&mut self, gfx_state: &GfxState, camera: &Camera) -> SpawnState {
        let light = &mut self.light;

        let mut light_spawner = gfx_state.create_spawner(SpawnOptions {
            camera,
            id: light.id.to_string(),
            emitter: light.emitter.clone(),
            light_layout: None,
        });

        while let Some(anim) = light.particle_animations.pop() {
            light_spawner.push_animation(anim.into_animation(gfx_state, &light_spawner));
        }

        while let Some(anim) = light.emitter_animations.pop() {
            light_spawner.push_emitter_animation(anim);
        }

        light_spawner
    }

    pub fn create_spawners(
        self,
        gfx_state: &GfxState,
        light_layout: &wgpu::BindGroupLayout,
        camera: &Camera,
    ) -> Vec<SpawnState> {
        let mut spawners: Vec<SpawnState> = Vec::new();

        for item in self.spawners {
            let mut spawner = gfx_state.create_spawner(SpawnOptions {
                camera,
                id: item.id,
                emitter: item.emitter,
                light_layout: Some(light_layout),
            });

            for anim in item.particle_animations {
                spawner.push_animation(anim.into_animation(gfx_state, &spawner));
            }

            for anim in item.emitter_animations {
                spawner.push_emitter_animation(anim);
            }

            let is_unique = spawners.iter().all(|s| spawner.id != s.id);

            assert!(!spawner.id.is_empty(), "Id can not be empty");
            assert!(is_unique, "Spawners requires an unique ID");

            spawners.push(spawner);
        }

        spawners
    }
}
