use super::event::{Event, EventType};
use std::fmt::{Debug, Error, Formatter};

pub type ListenerHandle = u64;

pub struct Listener<'a> {
    pub event: EventType,
    pub handle: ListenerHandle,
    callback: Box<FnMut(Event) + 'a>,
}

impl<'a> PartialEq for Listener<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl<'a> Debug for Listener<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Listener {{ event: {:?}, handle: {:?} callback: FnMut(Event) }}", self.event, self.handle)
    }
}

impl<'a> Listener<'a> {
    pub fn new<F>(handle: ListenerHandle, event_type: EventType, listener: F) -> Listener<'a>
        where F: FnMut(Event) + 'a
    {
        Listener { event: event_type, handle: handle, callback: Box::new(listener) }
    }

    pub fn call(&mut self, event: Event) {
        let ref mut func = self.callback;
        func(event);
    }

    pub fn has_handle(&self, handle: ListenerHandle) -> bool {
        self.handle == handle
    }
}
