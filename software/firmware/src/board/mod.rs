#[cfg(feature = "blackpill")]
pub use blackpill::*;
#[cfg(feature = "bluepill")]
pub use bluepill::*;
#[cfg(feature = "feather_nrf52840")]
pub use feather_nrf52840::*;
#[cfg(feature = "flightcontroller")]
pub use flightcontroller::*;

#[cfg(feature = "blackpill")]
mod blackpill;
#[cfg(feature = "bluepill")]
mod bluepill;
#[cfg(feature = "feather_nrf52840")]
mod feather_nrf52840;
#[cfg(feature = "flightcontroller")]
mod flightcontroller;

pub trait EnginePwm {
    fn get_max_duty(&self) -> u16;
    fn set_duty(&mut self, duty: [u16; 4]);
}

pub trait InterruptHandler {
    fn activate_radio_irq(&mut self);
    fn reset_radio_irq(&mut self);
}
