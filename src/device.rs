use crate::{
    hackrf_error_HACKRF_TRUE,
    hackrf_error_HACKRF_SUCCESS,
    hackrf_error_HACKRF_ERROR_STREAMING_STOPPED,
    hackrf_device,
    hackrf_is_streaming,
    hackrf_set_baseband_filter_bandwidth,
    hackrf_set_freq,
    hackrf_set_sample_rate_manual,
    hackrf_set_amp_enable,
    hackrf_set_lna_gain,
    hackrf_set_vga_gain,
    hackrf_set_txvga_gain,
    hackrf_set_antenna_enable,
    hackrf_set_hw_sync_mode
};

use crate::error::Error;

/// A HackRF device
#[derive(Debug)]
pub struct Device {
    pub(super) device: *mut hackrf_device
}

impl Device {
//    pub fn start_rx(&self, sample_block_cb_fn callback, void* rx_ctx) -> Result<(), Error> {}
//    pub fn stop_rx(&self) -> Result<(), Error> {}
//
//    pub fn start_tx(&self, sample_block_cb_fn callback, void* tx_ctx) -> Result<(), Error> {}
//    pub fn stop_tx(&self) -> Result<(), Error> {}
//
//    pub fn init_sweep(&self, const uint16_t* frequency_list, num_ranges: isize, num_bytes: u32, step_width: u32, offset: u32, const enum sweep_style style) -> Result<(), Error> {}

    /// Returns true if the device is streaming
    pub fn is_streaming(&self) -> Result<bool, Error> {
        unsafe {
            let ret = hackrf_is_streaming(self.device);

            if ret == hackrf_error_HACKRF_TRUE {
                Ok(true)
            } else if ret == hackrf_error_HACKRF_ERROR_STREAMING_STOPPED {
                Ok(false)
            } else {
                return Err(Error::from_code(ret));
            }
        }
    }

    /// Sets the baseband filter bandwidth
    pub fn set_baseband_filter_bandwidth(&self, bandwidth_hz: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_baseband_filter_bandwidth(self.device, bandwidth_hz);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }
        }

        Ok( () )
    }

//    pub fn board_id_read(&self, uint8_t* value) -> Result<(), Error> {}
//    pub fn version_string_read(&self, char* version, length: i8) -> Result<(), Error> {}
//    pub fn usb_api_version_read(&self, uint16_t* version) -> Result<(), Error> {}

    pub fn set_freq(&self, freq_hz: u64) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_freq(self.device, freq_hz);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }
        }

        Ok( () )
    }

//    pub fn set_freq_explicit(&self, if_freq_hz: u64, lo_freq_hz: u64, const enum rf_path_filter path) -> Result<(), Error> {}

    /* currently 8-20Mhz - either as a fraction, i.e. freq 20000000hz divider 2 -> 10Mhz or as plain old 10000000hz (double)
        preferred rates are 8, 10, 12.5, 16, 20Mhz due to less jitter */
    pub fn set_sample_rate_manual(&self, freq_hz: u32, divider: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_sample_rate_manual(self.device, freq_hz, divider);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }
        }

        Ok( () )
    }

//    pub fn set_sample_rate(&self, const double freq_hz) -> Result<(), Error> {}

    /// Enable or disable the external amp
    pub fn set_amp_enable(&self, value: bool) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_amp_enable(self.device, if value {1} else {0});

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }
        }

        Ok( () )
    }

//    pub fn board_partid_serialno_read(&self, read_partid_serialno_t* read_partid_serialno) -> Result<(), Error> {}

    /* range 0-40 step 8d, IF gain in osmosdr  */
    pub fn set_lna_gain(&self, value: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_lna_gain(self.device, value);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }
        }

        Ok( () )
    }

    /* range 0-62 step 2db, BB gain in osmosdr */
    pub fn set_vga_gain(&self, value: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_vga_gain(self.device, value);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }
        }

        Ok( () )
    }

    /* range 0-47 step 1db */
    pub fn set_txvga_gain(&self, value: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_txvga_gain(self.device, value);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }
        }

        Ok( () )
    }

    /* antenna port power control */
    pub fn set_antenna_enable(&self, value: bool) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_antenna_enable(self.device, if value {1} else {0});

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }
        }

        Ok( () )
    }

    // TODO: Double-check this... value _might_ be wrong
    pub fn set_hw_sync_mode(&self, value: bool) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_hw_sync_mode(self.device, if value {1} else {0});

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }
        }

        Ok( () )

    }

}