use crate::{
    hackrf_device
};

/// A HackRF device
#[derive(Debug)]
pub struct Device {
    pub(super) device: *mut hackrf_device
}

