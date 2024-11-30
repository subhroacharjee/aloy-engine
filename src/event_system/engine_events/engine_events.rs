use crate::event_system::event::Event;

pub enum EngineEventCategory {
    Application,
    Window,
    Input,
    Keyboard,
    Mouse,
}

pub trait EngineEvent: Event {
    fn get_category(&self) -> EngineEventCategory;
    fn get_parent_category(&self) -> Option<EngineEventCategory>;
    fn has_event(name: String) -> bool;
}
