use crate::event_system::event::Event;

use super::engine_events::EngineEvent;

#[derive(Debug)]
pub enum InputEvent {}

impl Event for InputEvent {
    fn get_name(&self) -> String {
        match self {
            _ => "InputEvent".to_string(),
        }
    }

    fn get_data(&self) -> Option<crate::event_system::event::DynamicStore> {
        None
    }
}

impl EngineEvent for InputEvent {
    fn get_category(&self) -> super::engine_events::EngineEventCategory {
        super::engine_events::EngineEventCategory::Input
    }

    fn get_parent_category(&self) -> Option<super::engine_events::EngineEventCategory> {
        match self {
            _ => None,
        }
    }

    fn has_event(name: String) -> bool {
        match name {
            _ => false,
        }
    }
}
