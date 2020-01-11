use super::dazeus::{DaZeus, DaZeusClient};
use super::event::{Event, EventType};
use std::cell::RefCell;
use std::fmt::{Debug, Error, Formatter};
use std::io::{Read, Write};
use std::ops::DerefMut;

/// An identifier for unsubscribing an event listener.
pub type ListenerHandle = u64;

pub struct Listener<'a> {
    pub event: EventType,
    pub handle: ListenerHandle,
    #[allow(clippy::type_complexity)]
    callback: RefCell<Box<dyn FnMut(Event, &dyn DaZeusClient) + 'a>>,
}

impl<'a> PartialEq for Listener<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl<'a> Debug for Listener<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Listener {{ event: {:?}, handle: {:?} callback: FnMut(Event) }}",
            self.event, self.handle
        )
    }
}

impl<'a> Listener<'a> {
    pub fn new<F>(handle: ListenerHandle, event_type: EventType, listener: F) -> Listener<'a>
    where
        F: FnMut(Event, &dyn DaZeusClient) + 'a,
    {
        Listener {
            event: event_type,
            handle,
            callback: RefCell::new(Box::new(listener)),
        }
    }

    pub fn call<T: Read + Write>(&self, event: Event, dazeus: &DaZeus<T>) {
        let mut fbox = self.callback.borrow_mut();
        let func = fbox.deref_mut();
        func(event, dazeus as &dyn DaZeusClient);
    }

    pub fn has_handle(&self, handle: ListenerHandle) -> bool {
        self.handle == handle
    }
}
