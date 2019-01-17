use crate::{
    // const
    hackrf_error_HACKRF_TRUE,
    hackrf_error_HACKRF_SUCCESS,
    hackrf_error_HACKRF_ERROR_STREAMING_STOPPED,
    hackrf_error_HACKRF_ERROR_STREAMING_THREAD_ERR,
    // structs
    hackrf_device,
    hackrf_sample_block_cb_fn,
    sweep_style,
    rf_path_filter,
    read_partid_serialno_t,
    hackrf_transfer,
    // functions
    hackrf_start_rx,
    hackrf_stop_rx,
    hackrf_start_tx,
    hackrf_stop_tx,
    hackrf_init_sweep,
    hackrf_is_streaming,
    hackrf_set_baseband_filter_bandwidth,
    hackrf_board_id_read,
    hackrf_version_string_read,
    hackrf_usb_api_version_read,
    hackrf_set_freq,
    hackrf_set_freq_explicit,
    hackrf_set_sample_rate_manual,
    hackrf_set_sample_rate,
    hackrf_set_amp_enable,
    hackrf_board_partid_serialno_read,
    hackrf_set_lna_gain,
    hackrf_set_vga_gain,
    hackrf_set_txvga_gain,
    hackrf_set_antenna_enable,
    hackrf_set_hw_sync_mode
};

use crate::error::Error;

use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;
use std::slice;
use std::rc::Rc;

#[derive(Debug)]
pub enum State {
    IDLE,
    RECEIVING,
    TRANSMITTING
}

/// A HackRF device
#[derive(Debug)]
pub struct Device<'a> {
    pub(super) device_ptr: *mut hackrf_device,
    pub(super) state: State,
    phantom: PhantomData<&'a hackrf_device>
}

#[repr(C)]
struct CallbackContext<'a, T> {
    context: Option<&'a mut T>,
    function: &'a mut FnMut(&[u8], &Option<&mut T>) -> Error
}

impl <'a> Device<'a> {
    pub fn new(device: *mut hackrf_device) -> Device<'a> {
        Device {
            device_ptr: device,
            state: State::IDLE,
            phantom: PhantomData
        }
    }

    // wrapper function for start_rx
    unsafe extern "C" fn rx_callback<C>(transfer: *mut hackrf_transfer) -> i32
    where C: std::fmt::Debug {
        // construct a slice given the pointer and valid length
        let buffer :&[u8] = slice::from_raw_parts((*transfer).buffer, (*transfer).valid_length as usize);

        // construct the context, "casting" back to a CallbackContext
        let ctx = (*(*transfer).device).rx_ctx;
        let ctx  = &mut *(ctx as *mut Box<CallbackContext<C>>);

        // call the function, and convert the Error into an i32
        Into::into((ctx.function)(buffer, &ctx.context))
    }

    pub fn start_rx<T, F>(&mut self, mut callback: F, rx_ctx: Option<&mut T>) -> Result<(), Error>
    where F: FnMut(&[u8], &Option<&mut T>) -> Error,
    T: std::fmt::Debug {
        unsafe {
            // package up our context and function
            let mut ctx = Box::new(CallbackContext {
                context: rx_ctx,
                function: &mut callback
            });

            debug!("ctx.context: {:?}", ctx.context);

            // convert our context into a void*
//            let ctx = &mut ctx as *mut _ as *mut c_void;
            let ctx = Box::leak(ctx) as *mut _ as *mut c_void;

            debug!("start_rx ctx: {:?}", ctx);

            // call the underlying function
            let ret = hackrf_start_rx(self.device_ptr, Some(Device::rx_callback::<T>), ctx);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        // set the state so we can cleanup properly
        self.state = State::RECEIVING;

        Ok( () )
    }

    pub fn stop_rx(&mut self) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_stop_rx(self.device_ptr);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        // set the state so we don't erroneously cleanup
        self.state = State::IDLE;

        Ok( () )
    }

    pub fn start_tx(&mut self, callback: hackrf_sample_block_cb_fn, tx_ctx: *mut c_void) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_start_tx(self.device_ptr, callback, tx_ctx);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        // set the state so we can cleanup properly
        self.state = State::TRANSMITTING;

        Ok( () )
    }

    pub fn stop_tx(&mut self) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_stop_tx(self.device_ptr);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        // set the state so we don't erroneously cleanup
        self.state = State::IDLE;

        Ok( () )
    }

    /// Initialize sweep mode:
    /// * `frequency_list` a list of start/stop pairs of frequencies in MHz; must be 1 to 10 in the list.
    /// * `num_bytes` number of sample bytes to capture after each tuning.
    /// * `step_width` width in Hz of the tuning step.
    /// * `offset` number of Hz added to every tuning frequency. Use to select center frequency based on the expected usable bandwidth.
    /// * `sweep_mode`:
    ///   * `LINEAR` means `step_width` is added to the current frequency at each step.
    ///   * `INTERLEAVED` invokes a scheme in which each step is divided into two interleaved sub-steps, allowing the host to select the best portions of the FFT of each sub-step and discard the rest.
    pub fn init_sweep(&self, frequency_list: &[u16], num_bytes: u32, step_width: u32, offset: u32, style: sweep_style) -> Result<(), Error> {
        unsafe {
            let frequency_list_ptr = frequency_list.as_ptr();
            let ret = hackrf_init_sweep(self.device_ptr, frequency_list_ptr, frequency_list.len() as i32, num_bytes, step_width, offset, style);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    /// Returns true if the device is streaming
    pub fn is_streaming(&self) -> Result<bool, Error> {
        unsafe {
            let ret = hackrf_is_streaming(self.device_ptr);

            debug!("is_streaming: {:?}", ret);

            if ret == hackrf_error_HACKRF_TRUE {
                Ok(true)
            } else if ret == hackrf_error_HACKRF_ERROR_STREAMING_STOPPED || ret == hackrf_error_HACKRF_ERROR_STREAMING_THREAD_ERR {
                Ok(false)
            } else {
                return Err(Error::from(ret));
            }
        }
    }

    /// Sets the baseband filter bandwidth
    pub fn set_baseband_filter_bandwidth(&self, bandwidth_hz: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_baseband_filter_bandwidth(self.device_ptr, bandwidth_hz);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    pub fn board_id_read(&self) -> Result<u8, Error> {
        unsafe {
            let mut value : u8 = 0;
            let ret = hackrf_board_id_read(self.device_ptr, &mut value);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            } else {
                return Ok(value);
            }
        }
    }

    pub fn version_string_read(&self) -> Result<String, Error> {
        unsafe {
            let buff = Vec::<u8>::with_capacity(255); // one less than max so there is space for a null
            let length = buff.capacity();
            let version = CString::new(buff).expect("Error creating string buffer");
            let version_ptr = version.into_raw();

            let ret = hackrf_version_string_read(self.device_ptr, version_ptr, length as u8);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            } else {
                return Ok(String::from(CString::from_raw(version_ptr).to_str().expect("Error converting string")));
            }
        }
    }

    pub fn usb_api_version_read(&self) -> Result<u16, Error> {
        unsafe {
            let mut value : u16 = 0;
            let ret = hackrf_usb_api_version_read(self.device_ptr, &mut value);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            } else {
                return Ok(value);
            }
        }
    }

    pub fn set_freq(&self, freq_hz: u64) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_freq(self.device_ptr, freq_hz);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    /// Sets the intermediate frequency (`if_freq_hz`) and local oscillator (`lo_freq_hz`) explicitly
    /// * `if_freq_hz` - must be in the range [2150000000, 2750000000]
    /// * `lo_freq_hz` - must be in the range [84375000, 5400000000]
    pub fn set_freq_explicit(&self, if_freq_hz: u64, lo_freq_hz: u64, path: rf_path_filter) -> Result<(), Error> {
        if if_freq_hz < 2150000000 {
            let err_str = format!("if_freq_hz {} < 2150000000", if_freq_hz);
            return Err(Error::INVALID_PARAM(err_str));
        } else if if_freq_hz > 2750000000 {
            let err_str = format!("if_freq_hz {} > 2750000000", if_freq_hz);
            return Err(Error::INVALID_PARAM(err_str));
        } else if lo_freq_hz < 84375000 {
            let err_str = format!("lo_freq_hz {} < 84375000", if_freq_hz);
            return Err(Error::INVALID_PARAM(err_str));
        } else if lo_freq_hz > 5400000000 {
            let err_str = format!("lo_freq_hz {} > 5400000000", if_freq_hz);
            return Err(Error::INVALID_PARAM(err_str));
        }

        unsafe {
            let ret = hackrf_set_freq_explicit(self.device_ptr, if_freq_hz, lo_freq_hz, path);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    /// You should probably use `set_sample_rate` below instead of this function.
    /// They both result in automatic baseband filter selection as described below.
    pub fn set_sample_rate_manual(&self, freq_hz: u32, divider: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_sample_rate_manual(self.device_ptr, freq_hz, divider);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    /// For anti-aliasing, the baseband filter bandwidth is automatically set to the
    /// widest available setting that is no more than 75% of the sample rate.  This
    /// happens every time the sample rate is set.  If you want to override the
    /// baseband filter selection, you must do so after setting the sample rate.
    pub fn set_sample_rate(&self, freq_hz: f64) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_sample_rate(self.device_ptr, freq_hz);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    /// Enable or disable the external amp
    pub fn set_amp_enable(&self, value: bool) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_amp_enable(self.device_ptr, if value {1} else {0});

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    pub fn board_partid_serialno_read(&self) -> Result<read_partid_serialno_t, Error> {
        unsafe {
            let mut value : read_partid_serialno_t = read_partid_serialno_t { part_id: [0; 2usize], serial_no: [0; 4usize]};
            let ret = hackrf_board_partid_serialno_read(self.device_ptr, &mut value);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            } else {
                return Ok(value);
            }
        }
    }

    /* range 0-40 step 8d, IF gain in osmosdr  */
    pub fn set_lna_gain(&self, value: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_lna_gain(self.device_ptr, value);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    /* range 0-62 step 2db, BB gain in osmosdr */
    pub fn set_vga_gain(&self, value: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_vga_gain(self.device_ptr, value);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    /* range 0-47 step 1db */
    pub fn set_txvga_gain(&self, value: u32) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_txvga_gain(self.device_ptr, value);

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    /* antenna port power control */
    pub fn set_antenna_enable(&self, value: bool) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_antenna_enable(self.device_ptr, if value {1} else {0});

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }

    /// Enable or disable hardware sync mode
    pub fn enable_hardware_sync(&self, enable: bool) -> Result<(), Error> {
        unsafe {
            let ret = hackrf_set_hw_sync_mode(self.device_ptr, if enable {1} else {0});

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from(ret));
            }
        }

        Ok( () )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hackrf::HackRF;
    use crate::LOGGER_INIT;
    use std::thread;

    #[test]
    fn start_stop_rx() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let mut dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        let ctx = None::<&mut i32>;

        dev.start_rx(|b, c| {
//            println!("BUFFER LEN: {}", b.len());
//            println!("CONTEXT: {:?}", c);

            Error::SUCCESS
        }, ctx).expect("Error calling start_rx");

        thread::sleep_ms(250);

        dev.stop_rx().expect("Error calling stop_rx");
    }


    #[test]
    fn is_streaming() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.is_streaming().unwrap(), "Should not be streaming");
    }

    #[test]
    fn set_baseband_filter_bandwidth() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.set_baseband_filter_bandwidth(200).is_err());
    }

    #[test]
    fn board_id_read() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        let id = dev.board_id_read().expect("Error calling board_id_read");

        assert_ne!(id, 0, "Board ID is zero");
    }

    #[test]
    fn version_string_read() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        let ver = dev.version_string_read().expect("Error calling version_string_read");

        assert_ne!(ver.len(), 0, "Version string has no length");
    }

    #[test]
    fn usb_api_version_read() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        let ver = dev.usb_api_version_read().expect("Error calling usb_api_version_read");

        assert_ne!(ver, 0, "USB API version is zero");
    }

    #[test]
    fn set_freq() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.set_freq(200).is_err());
    }

    #[test]
    fn set_freq_explicit() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        dev.set_freq_explicit(2160000000, 84385000, rf_path_filter::RF_PATH_FILTER_BYPASS).expect("set_freq_explicit failed");
    }

    #[test]
    fn set_sample_rate_manual() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.set_sample_rate_manual(2000, 25).is_err());
    }

    #[test]
    fn set_sample_rate() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.set_sample_rate(2000.0).is_err());
    }

    #[test]
    fn set_amp_enable() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.set_amp_enable(false).is_err());
    }

    #[test]
    fn board_partid_serialno_read() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        dev.board_partid_serialno_read().expect("Error calling board_partid_serialno_read");
    }

    #[test]
    fn set_lna_gain() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.set_lna_gain(5).is_err());
    }

    #[test]
    fn set_vga_gain() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.set_vga_gain(5).is_err());
    }

    #[test]
    fn set_txvga_gain() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.set_txvga_gain(5).is_err());
    }

    #[test]
    fn set_antenna_enable() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.set_antenna_enable(true).is_err());
    }

    #[test]
    fn set_hw_sync_mode() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let mut hrf = HackRF::new().expect("Error creating HackRF");
        let dev = hrf.open_device(0).expect("Error creating device; maybe not plugged in?");

        assert!(!dev.enable_hardware_sync(true).is_err());
    }

}