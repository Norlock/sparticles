use super::angles::Angles;
use super::particle::Particle;
use crate::clock::Clock;
use std::time::Duration;

pub struct EmitterSize {
    pub length: f32,
    pub depth: f32,
}

pub struct Emitter {
    pub emitter_position: cgmath::Vector3<f32>,
    pub emitter_size: EmitterSize,

    pub particles_per_emission: u32,
    pub delay_between_emission_ms: u32,
    pub particle_color: cgmath::Vector4<f32>,

    /// Newton force
    pub particle_speed: f32,
    /// number between 0 and 1, e.g. 0.001
    pub particle_friction_coefficient: f32,

    pub particle_size: f32,
    pub iteration: u32,

    pub emitter_duration: Duration,
    pub angle_degrees: Angles,

    /// Initial spread factor x,y / z
    pub diffusion_degrees: Angles,

    pub emission_distortion: f32,
    //pub particle_texture: Option<Texture2D>,
    pub particle_lifetime: Duration,

    pub particle_radius: f32,
    pub particle_mass: f32,

    pub bounds: Option<Bounds>,
    //pub particle_animation_options: Option<AnimationOptions>,
    //pub emitter_animation_handler: Option<EmitterAnimationHandler>,
    //pub force_handler: Option<ForceHandler>,
}

pub struct Bounds {
    pub start_x: Option<f32>,
    pub start_y: Option<f32>,
    pub start_z: Option<f32>,
    pub end_x: Option<f32>,
    pub end_y: Option<f32>,
    pub end_z: Option<f32>,
}

pub struct SpawnData<'a> {
    pub clock: &'a Clock,
    pub particles: &'a mut Vec<Particle>,
}

impl Emitter {
    pub fn spawn(&mut self, data: &mut SpawnData) {
        let elapsed_ms = data.clock.lifetime_ms() as u32;
        let new_iteration = elapsed_ms / self.delay_between_emission_ms;

        if self.iteration == new_iteration {
            return;
        }

        self.iteration = new_iteration;

        let velocity = cgmath::Vector3::new(self.particle_speed, self.particle_speed, 0.);

        //let mut rng = thread_rng();

        for _ in 0..self.particles_per_emission {
            //let emitter_length = gen_abs_range(&mut rng, emit_options.emitter_size.length);
            //let emitter_depth = gen_abs_range(&mut rng, emit_options.emitter_size.depth);
            //let distortion = gen_dyn_range(&mut rng, emit_options.emission_distortion);

            //let Angles { elevation, bearing } = emit_options.angle_radians;
            //// Used to emit perpendicular of emitter.
            //let perpendicular = elevation.cos() * -1.;
            //let x = distortion + emitter_length * perpendicular * bearing.cos();
            //let y = distortion + emitter_length * elevation.sin() * bearing.cos();
            //let z = (distortion + emitter_depth) + emitter_length * bearing.sin();

            //let diffusion_elevation_delta =
            //gen_dyn_range(&mut rng, emit_options.diffusion_radians.elevation);
            //let bearing_radians = gen_dyn_range(&mut rng, emit_options.diffusion_radians.bearing);
            //let elevation_radians =
            //emit_options.angle_emission_radians() + diffusion_elevation_delta;

            //// Used to emit perpendicular of emitter.
            //let perpendicular = elevation_radians.cos() * -1.;
            //let vx = particle_attributes.speed * perpendicular * bearing_radians.cos();
            //let vy = particle_attributes.speed * elevation_radians.sin() * bearing_radians.cos();
            //let vz = particle_attributes.speed * bearing_radians.sin();

            //let speed = Velocity { vx, vy, vz };
            //let life_cycle = LifeCycle {
            //spawned_at: total_elapsed_ms,
            //duration_ms: particle_attributes.duration_ms,
            //iteration: -1,
            //};

            //let attributes = ParticleAttributes {
            //friction_coefficient: particle_attributes.friction_coefficient,
            //radius: particle_attributes.radius,
            //mass: particle_attributes.mass,
            //color: particle_attributes.color.as_rgba_f32(),
            //};
            data.particles.push(Particle {
                position: self.emitter_position,
                color: self.particle_color,
                velocity,
                size: self.particle_size,
            })
        }
    }
}
