//! Motor management.

use std::{ops::RangeInclusive, time::Duration};

use crate::{
    actix::*,
    pin::{Error as PinError, Pin, Pwm},
};

/// A message that can be sent to a motor to change its position.
#[derive(Clone, Copy, Debug)]
pub enum Message {
    /// Requests that the motor be set to the closed position.
    Close,
    /// Requests that the motor be set to the open position.
    Open,
    /// Requests that the motor be set to the shut (not closed) position.
    Shut,
    /// Turns off the motor's output signal.
    Stop,
}

impl ActixMessage for Message {
    type Result = ();
}

/// A motor connected to the syringe manifold.
///
/// Moving a motor (physically) will cause the control knob to rotate.
#[derive(Debug)]
pub struct Motor {
    /// The characteristic period of the motor.
    period: Duration,
    /// The output pin controlling the physical motor.
    pin: Pin,
    /// The range of acceptable signal lengths.
    ///
    /// The motor is assumed to have 180º of motion, meaning the minimum and signals should
    /// correspond to antiparallel positions.
    ///
    /// The closed position is assumed to be 0º; the open position is at 90º.
    signal_range: RangeInclusive<Duration>,
    /// The duration for which the signal should be high in each period.
    ///
    /// Changing this property will change the position of the motor.
    pulse_width: Duration,
    /// The handle to the main loop for this motor (for cancellation).
    main_handle: Option<SpawnHandle>,
}

impl PartialEq for Motor {
    fn eq(&self, other: &Self) -> bool {
        self.pin.number == other.pin.number
    }
}

impl Eq for Motor {}

impl Motor {
    fn set_pulse_width(&mut self, width: Duration) -> Result<(), PinError> {
        log::debug!(
            "Setting pulse width of motor on pin {} to {:?}",
            self.pin.number,
            width
        );
        self.pulse_width = width;
        self.pin.set_pwm(self.period, width)
    }

    /// Sets the motor's angle in degrees (relative to the closed position).
    ///
    /// ## Panics
    /// This method will panic if `angle` is greater than 180.
    pub fn set_angle(&mut self, angle: u16) -> Result<(), PinError> {
        assert!(angle <= 180);
        let (start, end) = (self.signal_range.start(), self.signal_range.end());
        // Dereference, since auto-deref doesn't seem to work for std::ops::Sub?
        let (start, end) = (*start, *end);
        let delta = end - start;
        // Assume a range of motion of 180º.
        let range = 180;
        // Calculate the change in signal per unit angle (dT/dθ).
        let step = delta / range;
        // Multiply the step by the desired angle to get the offset from the baseline (∆T).
        let offset = step * angle.into();
        log::trace!(
            "Setting motor angle to {} (pulse width: {:?})",
            angle,
            start + offset
        );
        self.set_pulse_width(start + offset)
    }
    /// Sets the motor to the closed position (angle of 90º).
    ///
    /// Fluid will flow through the valve, but not from the associated buffer.
    pub fn close(&mut self) -> Result<(), PinError> {
        log::trace!("Closing motor on pin {}.", self.pin.number);
        self.set_angle(90)
    }
    /// Sets the motor to the shut position, where no fluid will flow through it.
    pub fn shut(&mut self) -> Result<(), PinError> {
        log::trace!("Shutting motor on pin {}.", self.pin.number);
        self.set_angle(180)
    }
    /// Sets the motor to the open position (angle of 0º).
    ///
    /// Fluid from the associated buffer will flow through the valve.
    pub fn open(&mut self) -> Result<(), PinError> {
        log::trace!("Opening motor on pin {}.", self.pin.number);
        self.set_angle(0)
    }
    ///
    /// Constructs a new motor with the given period and signal range on the given pin number, if
    /// possible.
    ///
    /// The motor will be set to the closed position initially.
    pub fn try_new<R>(period: Duration, range: R, pin: u16) -> Result<Self, PinError>
    where
        R: Into<RangeInclusive<Duration>>,
    {
        let pin = Pin::try_new(pin)?;
        let signal_range = range.into();
        Ok(Self {
            period,
            pin,
            pulse_width: *signal_range.start(),
            signal_range,
            main_handle: None,
        })
    }
    /// Constructs a new motor with the given period and signal range on the given pin number.
    ///
    /// The motor will be set to the closed position initially.
    ///
    /// ## Panics
    /// This method will panic if opening the pin fails. For a fallible initializer, see
    /// [`Motor::try_new`](#method.try_new).
    pub fn new<R>(period: Duration, range: R, pin: u16) -> Self
    where
        R: Into<RangeInclusive<Duration>>,
    {
        Self::try_new(period, range, pin).expect("Motor construction failed.")
    }
}

impl Actor for Motor {
    type Context = Context<Self>;
}

impl Handle<Message> for Motor {
    type Result = ();
    fn handle(&mut self, message: Message, _context: &mut Self::Context) -> Self::Result {
        match message {
            Message::Open => self.open().unwrap(),
            Message::Close => self.close().unwrap(),
            Message::Shut => self.shut().unwrap(),
            Message::Stop => {
                log::trace!("Stopping motor motion.");
                self.set_pulse_width(Duration::new(0, 0)).unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // This test makes sure the panic in validate_motor_angle isn't from constructing the motor and unwrapping it.
    #[test]
    fn make_fake_motor() {
        let _motor = Motor::try_new(
            Duration::new(2, 0),
            Duration::new(0, 0)..=Duration::new(1, 0),
            1,
        )
        .unwrap();
    }
    #[test]
    #[should_panic]
    fn validate_motor_angle() {
        let mut motor = Motor::try_new(
            Duration::new(2, 0),
            Duration::new(0, 0)..=Duration::new(1, 0),
            1,
        )
        .unwrap();
        let _ = motor.set_angle(181);
    }
}
