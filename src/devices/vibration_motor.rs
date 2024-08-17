use esp_hal::{
    gpio::{Level, Output, OutputPin},
    peripheral::Peripheral,
};

pub struct VibrationMotor<'d, P> {
    vib_pwm: Output<'d, P>,
}

impl<'d, P: OutputPin> VibrationMotor<'d, P> {
    pub fn new(pin: impl Peripheral<P = P> + 'd) -> Self {
        Self {
            vib_pwm: Output::new(pin, Level::Low),
        }
    }

    pub fn is_vibrating(&mut self) -> bool {
        self.vib_pwm.is_set_high()
    }

    pub fn set_vibrating(&mut self, vibrate: bool) {
        if vibrate {
            self.vib_pwm.set_high()
        } else {
            self.vib_pwm.set_low()
        }
    }
}
