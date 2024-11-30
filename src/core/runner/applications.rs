use core::panic;
use std::{
    process::exit,
    sync::{Arc, Mutex},
};

use log::{error, info, trace};

use crate::{
    core::runner::exit_handlers,
    event_system::{
        engine_events::application_events::ApplicationEvents,
        event::Event,
        event_dispatcher::{EventDispatcher, EventDispatcherErrors},
        event_queue::{self, EventQueueErrors},
    },
};

use super::exit_handlers::ExitReason;

#[derive(Debug, Default)]
pub struct Application {
    exit_flag: Arc<Mutex<Option<ExitReason>>>,
    dispatchers: Vec<EventDispatcher>,
}

impl Application {
    fn initalize(&mut self) {
        let exit_event = "Exit".to_string();
        let mut exit_flag = Arc::clone(&self.exit_flag);
        if let Some(err) = self.on_event(exit_event, move |e| {
            if let Some(exit) = e.get_data().unwrap().get_ref::<ExitReason>() {
                if let Ok(mut exit_flag) = exit_flag.try_lock() {
                    exit_flag.replace(exit.clone());
                }
            }
        }) {
            error!("error during initalization {:?}", err);
            panic!("error in initalization");
        }
    }

    pub fn on_event(
        &mut self,
        event_name: String,
        cb: impl Fn(&dyn Event) + Send + Sync + 'static,
    ) -> Option<EventDispatcherErrors> {
        let mut dispatcher = EventDispatcher::new(event_name);
        if let Err(err) = dispatcher.add_handlers(Arc::new(cb)) {
            return Some(err);
        }
        self.dispatchers.push(dispatcher);
        None
    }

    // For immdidate dispatching events
    pub fn dispatch(&self, event: &dyn Event) {
        for dispatcher in &self.dispatchers {
            if let Err(err) = dispatcher.dispatch(event) {
                error!("error in dispatch::{:?}", err);
            }
        }
    }

    pub fn run(&mut self) {
        info!("Start");

        let event_loop = event_queue::EventQueue::initalize();

        self.initalize();
        loop {
            // At every event cycle we will fetch all the events
            match event_loop.get_events() {
                Ok(events) => {
                    for event in events.iter() {
                        let e = event.as_ref();
                        self.dispatch(e);
                    }
                }
                Err(EventQueueErrors::EmptyQueue) => {
                    info!("No events in the global queue");
                }
                Err(EventQueueErrors::UnableToFetchEventsFromQueue) => {}
                _ => {}
            }

            {
                let exit_flag = Arc::clone(&self.exit_flag);
                if let Ok(mut exit_reason) = exit_flag.try_lock() {
                    if let Some(flag) = exit_reason.take() {
                        match flag {
                            ExitReason::NORMAL => {
                                exit(0);
                            }
                            ExitReason::ERROR(code) => exit(code),
                        }
                    }
                };
            }
            trace!("working");
        }
    }
}
