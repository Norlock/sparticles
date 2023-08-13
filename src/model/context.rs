use super::{Camera, Clock, GuiState, SpawnGuiState, SpawnState};

pub enum SparticleEvent {
    ResetCamera,
    Recreate,
}

pub struct SparticleContext {
    events: Vec<SparticleEvent>,
}

impl SparticleContext {
    pub fn new() -> Self {
        Self {
            components: vec![],
            events: vec![],
        }
    }

    pub fn push_component(&mut self, component: Component) {
        self.components.push(component);
    }

    pub fn get_clock(&self) -> Option<&Clock> {
        for component in self.components.iter() {
            if let Component::Clock(clock) = component {
                return Some(clock);
            }
        }
        return None;
    }

    pub fn get_clock_mut(&mut self) -> Option<&mut Clock> {
        for component in self.components.iter_mut() {
            if let Component::Clock(clock) = component {
                return Some(clock);
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Camera, Clock};

    use super::{Component, SparticleContext};

    #[test]
    fn it_works() {
        let mut ctx = SparticleContext::new();

        let clock = Clock::new();

        ctx.push_component(Component::Clock(clock));

        retrieve(&mut ctx);
    }

    fn retrieve(ctx: &mut SparticleContext) {
        let clock = ctx.get_clock();
        let clock2j = ctx.get_clock();

        if let Some(clock) = clock {
            println!("{}", clock.elapsed_sec());
        }

        //let clock = ctx.get_first::<Camera>();
        //let test = clock.get_mut();
    }
}
