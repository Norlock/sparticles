use std::any::Any;

pub struct SparticleContext {
    components: Vec<Box<dyn Any>>,
}

impl SparticleContext {
    pub fn new() -> Self {
        Self { components: vec![] }
    }

    pub fn push(&mut self, component: impl Any) {
        self.components.push(Box::new(component));
    }

    pub fn get<'a, T>(&self) -> Vec<&T>
    where
        T: Any,
    {
        self.components
            .iter()
            .filter(|c| c.is::<T>())
            .map(|c| c.downcast_ref::<T>())
            .map(|c| c.expect("Downcast failed"))
            .collect()
    }

    pub fn get_first<'a, T>(&self) -> &T
    where
        T: Any,
    {
        self.components
            .iter()
            .find(|c| c.is::<T>())
            .map(|c| c.downcast_ref::<T>())
            .map(|c| c.expect("Downcast failed"))
            .unwrap()
    }

    pub fn get_first_mut<'a, T>(&mut self) -> &mut T
    where
        T: Any,
    {
        self.components
            .iter_mut()
            .find(|c| c.is::<T>())
            .map(|c| c.downcast_mut::<T>())
            .map(|c| c.expect("Downcast failed"))
            .unwrap()
    }

    //pub fn get<'a, T>(&self) -> Vec<&T>
    //where
    //T: 'static + Component,
    //{
    //let type_id = TypeId::of::<T>();
    //let error_msg = format!("Cannot downcast: {}", type_name::<T>());

    //self.components
    //.iter()
    //.filter(|c| c.matches_type(&type_id))
    //.map(|c| c.get().downcast_ref::<T>())
    //.map(|c| c.expect(&error_msg))
    //.collect()
    //}

    //pub fn get_mut<'a, T>(&mut self) -> Vec<&mut T>
    //where
    //T: 'static + Component,
    //{
    //let type_id = TypeId::of::<T>();
    //let error_msg = format!("Cannot downcast: {}", type_name::<T>());

    //self.components
    //.iter_mut()
    //.filter(|c| c.matches_type(&type_id))
    //.map(|c| c.get_mut().downcast_mut::<T>())
    //.map(|c| c.expect(&error_msg))
    //.collect()
    //}

    //pub fn get_first<'a, T>(&'a self) -> &'a T
    //where
    //T: 'static + Component,
    //{
    //let type_id = TypeId::of::<T>();
    //let type_name = type_name::<T>();

    //self.components
    //.iter()
    //.find(|c| c.matches_type(&type_id))
    //.expect(&format!("Component: {:?} doesn't exist", type_name))
    //.get()
    //.downcast_ref::<T>()
    //.expect(&format!("Component: {:?} can't be casted", type_name))
    //}

    //pub fn get_first_mut<'a, T>(&'a mut self) -> &'a mut T
    //where
    //T: 'static + Component,
    //{
    //let type_id = TypeId::of::<T>();
    //let type_name = type_name::<T>();

    //self.components
    //.iter_mut()
    //.find(|c| c.matches_type(&type_id))
    //.expect(&format!("Component: {:?} doesn't exist", type_name))
    //.get_mut()
    //.downcast_mut::<T>()
    //.expect(&format!("Component: {:?} can't be casted", type_name))
    //}
}

#[cfg(test)]
mod tests {

    use crate::model::Clock;

    use super::SparticleContext;

    #[test]
    fn it_works() {
        let mut ctx = SparticleContext::new();

        let clock = Clock::new();

        ctx.push(clock);

        retrieve(&mut ctx);
    }

    fn retrieve(ctx: &mut SparticleContext) {
        let _clock = ctx.get_first::<Clock>();

        //println!("elapsed {}", clock.elapsed_sec());

        //let clock = ctx.get_first::<Camera>();
        //let test = clock.get_mut();
    }
}
