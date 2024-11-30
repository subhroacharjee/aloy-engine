use std::{any::Any, fmt::Debug};

#[derive(Debug)]
pub struct DynamicStore {
    value: Box<dyn Any>,
}

impl DynamicStore {
    pub fn new(value: Box<dyn Any>) -> Self {
        Self { value }
    }

    pub fn get_ref<T: 'static>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }
}

pub trait Event: Debug + Send + Sync {
    fn get_name(&self) -> String;
    fn get_data(&self) -> Option<DynamicStore>;
}
