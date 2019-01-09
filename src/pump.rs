//! Pump management.
use std::thread;

use crate::actix::*;
use crate::pin::Pin;

/// Messages that can be sent to the pump to change its direction or turn it off.
#[derive(Clone, Copy, Debug)]
pub enum Message {
    /// Asks the pump to run in the forward direction.
    Perfuse,
    /// Asks the pump to run in the backward direction.
    Drain,
    /// Asks the pump to stop.
    Stop,
}

impl ActixMessage for Message {
    type Result = ();
}

/// The direction of a pump.
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    /// The pump should run in the forward direction (toward the sample), perfusing any sample.
    Forward,
    /// The pump should run in the backward direction (toward waste), draining any sample.
    Backward,
}

/// Represents a pump.
///
/// ## Notes
/// The pump is assumed to operate using an [H-bridge](https://en.wikipedia.org/wiki/H_bridge), and
/// so requires four pins.
///
/// We assume there's a single pump elsewhere in the architecture, although this code could be used
/// to control multiple pumps concurrently.
///
/// ### Diagram
/// Here is a circuit diagram showing the meaning of each pin number.
/// Each pin controls a relay/transistor in the H-bridge.
/// ```plaintext
///  +-----+-----+
///  |     0     1
/// +V     +-----+
///  |     2     3
///  +-----+-----+
/// ```
pub struct Pump {
    /// The GPIO pins to use for the H-bridge.
    pins: [Pin; 4],
    /// The direction the pump should run in (if running).
    direction: Option<Direction>,
}

impl Pump {
    /// Creates a new pump using the given GPIO pin numbers.
    pub fn new(_pins: [u8; 4]) -> Self {
        Self {
            direction: None,
            pins: unimplemented!(),
        }
    }
    /// Changes the pump direction to the specified direction.
    ///
    /// If the pump is not already stopped, it will be stopped and a wait of 20 ms will be added to
    /// prevent sparks, short-circuits, etc.
    pub fn set_direction<D>(&mut self, direction: D)
    where
        D: Into<Option<Direction>>,
    {
        let direction = direction.into();
        if let Some(direction) = direction {
            if !self.is_stopped() {
                self.stop();
                // Sleep to make sure we avoid Bad Things™️
                thread::sleep(std::time::Duration::from_millis(20));
            }
            let pins = match direction {
                Direction::Forward => (0, 3),
                Direction::Backward => (1, 2),
            };
            let (top, bottom) = (pins.0, pins.1);
            self.pins[top].set_high();
            self.pins[bottom].set_high();
        } else {
            for i in 0..4 {
                self.pins[i].set_low();
            }
        }
        self.direction = direction;
    }
    /// Switches the pump to the forward direction.
    pub fn perfuse(&mut self) {
        self.set_direction(Direction::Forward);
    }
    /// Switches the pump to the reverse direction.
    pub fn drain(&mut self) {
        self.set_direction(Direction::Backward);
    }
    /// Stops the pump.
    pub fn stop(&mut self) {
        self.set_direction(None);
    }
    /// Whether the pump is currently stopped.
    pub fn is_stopped(&self) -> bool {
        self.direction.is_none()
    }
}

impl Actor for Pump {
    type Context = Context<Self>;
}

impl Handle<Message> for Pump {
    type Result = ();
    fn handle(&mut self, message: Message, _context: &mut Self::Context) -> Self::Result {
        match message {
            Message::Perfuse => self.perfuse(),
            Message::Drain => self.drain(),
            Message::Stop => self.stop(),
        }
    }
}