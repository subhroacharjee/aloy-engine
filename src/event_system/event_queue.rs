use std::sync::{
    mpsc::{self, Receiver, SendError, Sender},
    Arc, Mutex,
};

use lazy_static::lazy_static;
use log::error;
use thiserror::Error;

use super::event::Event;

type BoxedEvent = Box<dyn Event>;

impl PartialEq for BoxedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.get_name() == other.get_name()
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum EventQueueErrors {
    #[error("unable to emit event in the event queue")]
    UnableToEmitToEventQueue(SendError<BoxedEvent>),

    #[error("unable to emit event in the event queue")]
    UnableToFetchEventsFromQueue,

    #[error("unable to emit event in the event queue")]
    EmptyQueue,
}

pub struct EventQueue {
    sender: Sender<BoxedEvent>,
    reciever: Arc<Mutex<Receiver<BoxedEvent>>>,
}

lazy_static! {
    static ref GLOBAL_EVENT_QUEUE: Arc<EventQueue> = Arc::new(EventQueue::new());
}

impl EventQueue {
    pub fn new() -> Self {
        let (sender, reciever) = mpsc::channel();
        Self {
            sender,
            reciever: Arc::new(Mutex::new(reciever)),
        }
    }

    pub fn initalize() -> Arc<EventQueue> {
        Arc::clone(&GLOBAL_EVENT_QUEUE)
    }

    pub fn emit(&self, event: Box<impl Event + 'static>) -> Result<(), EventQueueErrors> {
        if let Err(e) = self.sender.send(event) {
            return Err(EventQueueErrors::UnableToEmitToEventQueue(e));
        }
        Ok(())
    }

    pub fn get_events(&self) -> Result<Vec<BoxedEvent>, EventQueueErrors> {
        let mut events = Vec::new();
        match self.reciever.try_lock() {
            Ok(locked_recvr) => {
                while let Ok(event) = locked_recvr.try_recv() {
                    events.push(event);
                }

                if events.is_empty() {
                    return Err(EventQueueErrors::EmptyQueue);
                }

                Ok(events)
            }
            Err(err) => {
                error!(
                    "unable to lock the reciever of the channel. Failed with error, {:?}",
                    err
                );
                Err(EventQueueErrors::UnableToFetchEventsFromQueue)
            }
        }
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::event_system::event::DynamicStore;

    use super::*;

    #[derive(Debug)]
    struct TestEvent {
        name: String,
    }

    impl TestEvent {
        pub fn new(name: String) -> Self {
            TestEvent { name }
        }
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
    fn test_event_queue_constructor() {
        let queue = EventQueue::new();
        let event: BoxedEvent = Box::new(TestEvent::new(String::from_str("test event 1").unwrap()));
        let event_name = event.get_name();
        assert!(queue.sender.send(event).is_ok());

        {
            let lock = queue.reciever.try_lock();
            assert!(lock.is_ok());
            let rcvr = lock.unwrap();
            let mut counter = 0;
            while let Ok(event) = rcvr.try_recv() {
                counter += 1;
                assert_eq!(event.get_name(), event_name);
            }

            assert_ne!(counter, 0);
        }
    }

    #[test]
    fn test_event_queue_emit() {
        let queue = EventQueue::new();
        let event1 = TestEvent::new("Event 1".to_string());
        let event2 = TestEvent::new("Event 2".to_string());

        assert!(queue.emit(Box::new(event1)).is_ok());
        assert!(queue.emit(Box::new(event2)).is_ok());

        let events = queue.get_events();
        assert!(events.is_ok());

        let event_lists = events.unwrap();
        assert_eq!(event_lists.len(), 2);
        assert_eq!(event_lists.first().unwrap().get_name(), "Event 1");
        assert_eq!(event_lists.get(1).unwrap().get_name(), "Event 2");
    }

    #[test]
    fn test_event_queue_get_events_on_empty_queue() {
        let queue = EventQueue::new();

        let events = queue.get_events();
        assert!(events.is_err());
        if let Err(error) = events {
            match error {
                EventQueueErrors::EmptyQueue => {}
                _ => panic!("invalid error"),
            }
        }
    }

    #[test]
    fn test_emit_event_failure() {
        // Simulate a failure by dropping the sender
        let mut queue = EventQueue::new();
        let (sender, _) = mpsc::channel();

        // cant drop the sender inside event queue so, we reassign to test
        queue.sender = sender.clone();
        drop(sender);

        let result = queue.emit(Box::new(TestEvent::new("Event 1".to_string())));
        assert!(result.is_err());

        if let Err(error) = result {
            if let EventQueueErrors::UnableToEmitToEventQueue(err) = error {
                println!("err::{:?}", err)
            } else {
                panic!("Unexpected error type");
            }
        }
    }

    #[test]
    fn test_get_events_event_failure() {
        // Simulate a failure by locking the receiver
        let queue = EventQueue::new();

        let result = queue.emit(Box::new(TestEvent::new("Event 1".to_string())));
        assert!(result.is_ok());

        {
            let lock = queue.reciever.lock();
            assert!(lock.is_ok());

            let result = queue.get_events();
            assert!(result.is_err());

            if let Err(error) = result {
                if let EventQueueErrors::UnableToFetchEventsFromQueue = error {
                    // Expected error type
                } else {
                    panic!("Unexpected error type");
                }
            }
        }
    }
}
