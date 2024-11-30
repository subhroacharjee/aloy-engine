use std::{any::Any, i128};

use super::engine_events::EngineEvent;
use crate::{
    core::runner::exit_handlers::ExitReason,
    event_system::event::{DynamicStore, Event},
};

#[derive(Debug)]
pub enum ApplicationEvents {
    Exit(ExitReason),
    ExampleEvent,
    ExampleEventWithData(i128, i128),
}

impl EngineEvent for ApplicationEvents {
    fn get_category(&self) -> super::engine_events::EngineEventCategory {
        return super::engine_events::EngineEventCategory::Application;
    }

    fn get_parent_category(&self) -> Option<super::engine_events::EngineEventCategory> {
        match self {
            _ => None,
        }
    }

    fn has_event(name: String) -> bool {
        let n: &str = &name;
        match n {
            "ExampleEvent" | "ExampleEventWithData" | "Exit" => true,
            _ => false,
        }
    }
}

impl Event for ApplicationEvents {
    fn get_name(&self) -> String {
        match self {
            Self::ExampleEvent => "ExampleEvent".to_string(),
            Self::ExampleEventWithData(_, _) => "ExampleEventWithData".to_string(),
            Self::Exit(_) => "Exit".to_string(),
            _ => "ApplicationEvents".to_string(),
        }
    }

    fn get_data(&self) -> Option<crate::event_system::event::DynamicStore> {
        match self {
            Self::ExampleEventWithData(coord_x, coord_y) => {
                let coords = Box::new(vec![*coord_x, *coord_y]);
                let wrapped = coords as Box<dyn Any>;
                Some(DynamicStore::new(wrapped))
            }
            Self::Exit(exit) => {
                let exit_enum = Box::new(exit.clone());
                let wrapped = exit_enum as Box<dyn Any>;
                Some(DynamicStore::new(wrapped))
            }
            _ => None,
        }
    }
}
