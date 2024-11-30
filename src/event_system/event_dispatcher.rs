use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use log::{error, info, warn};
use thiserror::Error;

use super::event::Event;

pub type DispatcherCallback = Arc<dyn Fn(&dyn Event) + Send + Sync>;

#[derive(Debug, Error, PartialEq)]
pub enum EventDispatcherErrors {
    #[error("unable to add handler")]
    UnableToAddHandler,
}

pub struct EventDispatcher {
    event_name: String,
    handlers: Arc<Mutex<Vec<DispatcherCallback>>>,
}

impl EventDispatcher {
    pub fn new(event_name: String) -> Self {
        EventDispatcher {
            event_name,
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_handlers(&mut self, cb: DispatcherCallback) -> Result<(), EventDispatcherErrors> {
        let mut counter = 0;
        let event_name = self.event_name.to_string();
        info!("adding new handler for {}", event_name);
        loop {
            let lock = self.handlers.try_lock();
            match lock {
                Ok(mut handlers) => {
                    handlers.push(cb);
                    return Ok(());
                }
                Err(err) => {
                    error!("error in dispatch method: Error: {}", err);
                    if counter == 4 {
                        error!("{}'s event handler addition failed", event_name);
                        return Err(EventDispatcherErrors::UnableToAddHandler);
                    } else {
                        warn!(
                            "trying to lock handlers {} times for event {}",
                            counter, event_name
                        );
                        counter += 1;
                    }
                }
            }
        }
    }

    pub fn dispatch(&self, event: &dyn Event) -> Result<bool, EventDispatcherErrors> {
        let event_name = self.event_name.to_string();
        if event_name != event.get_name() {
            return Ok(false);
        }
        info!("dispatching all handlers for {}", event_name);
        let mut counter = 0;
        loop {
            let lock = self.handlers.try_lock();
            match lock {
                Ok(handlers) => {
                    handlers.clone().into_iter().for_each(|handler| {
                        handler(event);
                    });
                    return Ok(true);
                }
                Err(err) => {
                    error!("error in dispatch method: Error: {}", err);
                    if counter == 4 {
                        error!("{}'s event handler addition failed", event_name);
                        return Err(EventDispatcherErrors::UnableToAddHandler);
                    } else {
                        warn!(
                            "trying to lock handlers {} times for event {}",
                            counter, event_name
                        );
                        counter += 1;
                    }
                }
            }
        }
    }
}

impl Debug for EventDispatcher
// where
//     T: Event,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventDispatcher")
            .field("event_name", &self.event_name)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::atomic::AtomicU8};

    // use crate::core::logger::init_logger;

    use crate::event_system::event::DynamicStore;

    use super::*;

    #[derive(Debug)]
    struct TestEvent {
        name: String,
    }

    impl Event for TestEvent {
        fn get_name(&self) -> String {
            self.name.clone()
        }

        fn get_data(&self) -> Option<DynamicStore> {
            return None;
        }
    }

    #[test]
    fn test_add_handlers_success() {
        let test_event = TestEvent {
            name: String::from_str("Test Event").unwrap(),
        };

        let mut dispatcher = EventDispatcher::new(test_event.get_name());
        let handler_call_counter = Arc::new(AtomicU8::new(0));

        let callback = {
            let counter = Arc::clone(&handler_call_counter);
            Arc::new(move |_event: &dyn Event| {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            })
        };

        let result = dispatcher.add_handlers(callback);
        assert!(result.is_ok(), "Handler should be added successfully");

        dispatcher
            .dispatch(&test_event)
            .expect("Dispatch should succedd");

        assert_eq!(
            handler_call_counter.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "Handler should have been called exactly once"
        );
    }

    #[test]
    fn test_add_multiple_handlers_to_success() {
        let test_event = TestEvent {
            name: String::from_str("Test Event").unwrap(),
        };

        let mut dispatcher = EventDispatcher::new(test_event.get_name());
        let handler_call_counter = Arc::new(AtomicU8::new(0));

        let cb1 = {
            let counter = Arc::clone(&handler_call_counter);
            Arc::new(move |_event: &dyn Event| {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            })
        };

        let cb2 = {
            let counter = Arc::clone(&handler_call_counter);
            Arc::new(move |_event: &dyn Event| {
                counter.fetch_add(2, std::sync::atomic::Ordering::SeqCst);
            })
        };

        let cb3 = {
            let counter = Arc::clone(&handler_call_counter);
            Arc::new(move |_event: &dyn Event| {
                counter.fetch_add(3, std::sync::atomic::Ordering::SeqCst);
            })
        };

        let result = dispatcher.add_handlers(cb1);
        assert!(result.is_ok(), "Handler 1 should be added successfully");

        let result = dispatcher.add_handlers(cb2);
        assert!(result.is_ok(), "Handler 2 should be added successfully");

        let result = dispatcher.add_handlers(cb3);
        assert!(result.is_ok(), "Handler 3 should be added successfully");

        dispatcher
            .dispatch(&test_event)
            .expect("Dispatch should succedd");

        assert_eq!(
            handler_call_counter.load(std::sync::atomic::Ordering::SeqCst),
            6,
            "Handler should have been called exactly once"
        );
    }

    #[test]
    fn test_when_unknown_event_is_called_from_dispatcher() {
        let test_event = TestEvent {
            name: String::from_str("Test Event").unwrap(),
        };

        let mut dispatcher = EventDispatcher::new(test_event.get_name());
        let handler_call_counter = Arc::new(AtomicU8::new(0));

        let callback = {
            let counter = Arc::clone(&handler_call_counter);
            Arc::new(move |_event: &dyn Event| {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            })
        };

        let result = dispatcher.add_handlers(callback);
        assert!(result.is_ok(), "Handler should be added successfully");

        dispatcher
            .dispatch(&TestEvent {
                name: String::from_str("Some other event").unwrap(),
            })
            .expect("Dispatch should succedd");

        assert_eq!(
            handler_call_counter.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "Handler should have been called exactly once"
        );
    }

    #[test]
    fn test_handler_lock_failure() {
        // init_logger();
        let mut dispatcher = EventDispatcher::new("TestEvent".to_string());

        let handler_call_count = Arc::new(AtomicU8::new(0));
        let handler = {
            let counter = Arc::clone(&handler_call_count);
            Arc::new(move |_event: &dyn Event| {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            })
        };

        // Intentionally cause a deadlock by locking `handlers` manually
        {
            let handlers = dispatcher.handlers.clone();
            let mut _handlers_lock = handlers.lock().unwrap();
            let result = dispatcher.add_handlers(handler);

            assert!(
                result.is_err(),
                "Adding a handler should fail due to lock contention"
            );
            assert_eq!(
                result.unwrap_err(),
                EventDispatcherErrors::UnableToAddHandler,
                "Error should indicate inability to add handler"
            );
        }
    }
}
