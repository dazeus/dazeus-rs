use super::event::{Event, EventType};
use super::dazeus::DaZeus;
use std::fmt::{Debug, Error, Formatter};
use std::io::{Read, Write};
use std::cell::RefCell;
use std::ops::DerefMut;

/// An identifier for unsubscribing an event listener.
pub type ListenerHandle = u64;

pub struct Listener<'a, T> where T: Read + Write {
    pub event: EventType,
    pub handle: ListenerHandle,
    callback: RefCell<Box<FnMut(Event, &DaZeus<T>) + 'a>>,
}

impl<'a, T> PartialEq for Listener<'a, T> where T: Read + Write {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl<'a, T> Debug for Listener<'a, T> where T: Read + Write {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Listener {{ event: {:?}, handle: {:?} callback: FnMut(Event) }}", self.event, self.handle)
    }
}

impl<'a, T> Listener<'a, T> where T: Read + Write {
    pub fn new<F>(handle: ListenerHandle, event_type: EventType, listener: F) -> Listener<'a, T>
        where F: FnMut(Event, &DaZeus<T>) + 'a
    {
        Listener { event: event_type, handle: handle, callback: RefCell::new(Box::new(listener)) }
    }

    pub fn call(&self, event: Event, dazeus: &DaZeus<T>) {
        let mut fbox = self.callback.borrow_mut();
        let mut func = fbox.deref_mut();
        func(event, dazeus);
    }

    pub fn has_handle(&self, handle: ListenerHandle) -> bool {
        self.handle == handle
    }
}
