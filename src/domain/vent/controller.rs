use std::sync::Mutex;

use crate::domain::vent::vent::VentState;

pub struct VentController {
    state: Mutex<VentState>,
}

impl VentController {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(VentState::Closed),
        }
    }

    pub fn open(&self) {
        // here you’d set a GPIO pin high, or send PWM signal
        println!("Vent opening...");
        *self.state.lock().unwrap() = VentState::Open;
    }

    pub fn close(&self) {
        println!("Vent closing...");
        *self.state.lock().unwrap() = VentState::Closed;
    }

    pub fn status(&self) -> VentState {
        *self.state.lock().unwrap()
    }
}
