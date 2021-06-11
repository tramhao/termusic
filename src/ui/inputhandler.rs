use crossterm::event::{poll, read, Event};
use std::time::Duration;

pub struct InputHandler;

impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {}
    }

    pub fn read_event(&self) -> Result<Option<Event>, ()> {
        if let Ok(available) = poll(Duration::from_millis(100)) {
            match available {
                true => {
                    // Read event
                    if let Ok(ev) = read() {
                        Ok(Some(ev))
                    } else {
                        Err(())
                    }
                }
                false => Ok(None),
            }
        } else {
            Err(())
        }
    }
}
