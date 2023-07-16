use embassy_rp::pwm::{Channel, Config, Pwm as RpPwm};
pub enum PwmSlice {
    A,
    B,
}

pub struct Pwm<'d, T: Channel> {
    pwm: RpPwm<'d, T>,
    config: Config,
    slice: PwmSlice,
}

impl<'d, T: Channel> Pwm<'d, T> {
    pub fn new(pwm: RpPwm<'d, T>, config: Config, slice: PwmSlice) -> Self {
        Self { pwm, config, slice }
    }

    pub fn set_duty(&mut self, duty: u16) {
        match self.slice {
            PwmSlice::A => self.config.compare_a = duty,
            PwmSlice::B => self.config.compare_b = duty,
        }
        self.pwm.set_config(&self.config);
    }

    pub fn get_duty(&self) -> u16 {
        match self.slice {
            PwmSlice::A => self.config.compare_a,
            PwmSlice::B => self.config.compare_b,
        }
    }

    pub fn get_max_duty(&self) -> u16 {
        self.config.top
    }
}
