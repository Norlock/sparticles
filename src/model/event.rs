use std::any::{type_name, Any, TypeId};

use super::SpawnGuiState;

pub enum SparticleEvent {
    UpdateGui(SpawnGuiState),
    ResetCamera,
    Recreate,
}

pub trait Component {
    fn get(self: &Self) -> &dyn Any;
    fn get_mut(&mut self) -> &mut dyn Any;

    fn matches_type(&self, type_id: &TypeId) -> bool
    where
        Self: 'static,
    {
        self.type_id() == *type_id
    }
}

pub struct SparticleContext {
    components: Vec<Box<dyn Component>>,
}

impl SparticleContext {
    pub fn new() -> Self {
        Self { components: vec![] }
    }

    pub fn push(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }

    pub fn get<'a, T>(&self) -> Vec<&T>
    where
        T: 'static + Component,
    {
        let type_id = TypeId::of::<T>();
        let error_msg = format!("Cannot downcast: {}", type_name::<T>());

        self.components
            .iter()
            .filter(|c| c.matches_type(&type_id))
            .map(|c| c.get().downcast_ref::<T>())
            .map(|c| c.expect(&error_msg))
            .collect()
    }

    pub fn get_mut<'a, T>(&mut self) -> Vec<&mut T>
    where
        T: 'static + Component,
    {
        let type_id = TypeId::of::<T>();
        let error_msg = format!("Cannot downcast: {}", type_name::<T>());

        self.components
            .iter_mut()
            .filter(|c| c.matches_type(&type_id))
            .map(|c| c.get_mut().downcast_mut::<T>())
            .map(|c| c.expect(&error_msg))
            .collect()
    }

    pub fn get_first<'a, T>(&'a self) -> &'a T
    where
        T: 'static + Component,
    {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();

        self.components
            .iter()
            .find(|c| c.matches_type(&type_id))
            .expect(&format!("Component: {:?} doesn't exist", type_name))
            .get()
            .downcast_ref::<T>()
            .expect(&format!("Component: {:?} can't be casted", type_name))
    }

    pub fn get_first_mut<'a, T>(&'a mut self) -> &'a mut T
    where
        T: 'static + Component,
    {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();

        self.components
            .iter_mut()
            .find(|c| c.matches_type(&type_id))
            .expect(&format!("Component: {:?} doesn't exist", type_name))
            .get_mut()
            .downcast_mut::<T>()
            .expect(&format!("Component: {:?} can't be casted", type_name))
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Camera, Clock};

    use super::SparticleContext;

    #[test]
    fn it_works() {
        let mut ctx = SparticleContext::new();

        let clock = Clock::new();

        ctx.push(Box::new(clock));

        retrieve(&mut ctx);
    }

    fn retrieve(ctx: &mut SparticleContext) {
        let clock = ctx.get_first::<Clock>();
        println!("{}", clock.elapsed_sec());
        let clock = ctx.get_first::<Camera>();
        //let test = clock.get_mut();
    }
}
