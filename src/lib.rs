#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use]
extern crate log;

use std::collections::HashSet;
use std::fmt;
use std::ffi::CStr;
use std::mem;
use std::ptr;

#[cfg(test)] use std::sync::{Once, ONCE_INIT};
#[cfg(test)] static LOGGER_INIT: Once = ONCE_INIT;


///
/// Conversion of enum hackrf_error that includes the result of a call to hackrf_error_name
///
#[derive(Debug)]
pub enum Error {
    SUCCESS,
    TRUE,
    INVALID_PARAM(String),
    NOT_FOUND(String),
    BUSY(String),
    NO_MEMORY(String),
    LIBUSB(String),
    THREAD(String),
    STREAMING_THREAD_ERR(String),
    STREAMING_STOPPED(String),
    STREAMING_EXIT_CALLED(String),
    USB_API_VERSION(String),
    NOT_LAST_DEVICE(String),
    OTHER(String)
}

impl  fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SUCCESS => write!(f, "SUCCESS"),
            Error::TRUE => write!(f, "TRUE"),
            Error::INVALID_PARAM(s) => write!(f, "INVALID PARAM: {}", s),
            Error::NOT_FOUND(s) => write!(f, "NOT FOUND: {}", s),
            Error::BUSY(s) => write!(f, "BUSY: {}", s),
            Error::NO_MEMORY(s) => write!(f, "NO MEMORY: {}", s),
            Error::LIBUSB(s) => write!(f, "LIBUSB: {}", s),
            Error::THREAD(s) => write!(f, "THREAD: {}", s),
            Error::STREAMING_THREAD_ERR(s) => write!(f, "STREAMING THREAD ERR: {}", s),
            Error::STREAMING_STOPPED(s) => write!(f, "STREAMING STOPPED: {}", s),
            Error::STREAMING_EXIT_CALLED(s) => write!(f, "STREAMING EXIT CALLED: {}", s),
            Error::USB_API_VERSION(s) => write!(f, "USB API VERSION: {}", s),
            Error::NOT_LAST_DEVICE(s) => write!(f, "NOT LAST DEVICE: {}", s),
            Error::OTHER(s) => write!(f, "OTHER: {}", s),
        }
    }
}

impl  Error {
    fn get_error_string(error_code: i32) -> String {
        unsafe {
            let char_ptr = hackrf_error_name(error_code);
            let c_str = CStr::from_ptr(char_ptr);


            return String::from(c_str.to_str().expect("Error converting hackrf_error_name"));
        }
    }

    pub fn from_code(error_code: i32) -> Error {
        match error_code {
            hackrf_error_HACKRF_SUCCESS => Error::SUCCESS,
            hackrf_error_HACKRF_TRUE => Error::TRUE,
            hackrf_error_HACKRF_ERROR_INVALID_PARAM => Error::INVALID_PARAM(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_NOT_FOUND => Error::NOT_FOUND(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_BUSY => Error::BUSY(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_NO_MEM => Error::NO_MEMORY(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_LIBUSB => Error::LIBUSB(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_THREAD => Error::THREAD(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_STREAMING_THREAD_ERR => Error::STREAMING_THREAD_ERR(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_STREAMING_STOPPED => Error::STREAMING_STOPPED(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_STREAMING_EXIT_CALLED => Error::STREAMING_EXIT_CALLED(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_USB_API_VERSION => Error::USB_API_VERSION(Error::get_error_string(error_code)),
            hackrf_error_HACKRF_ERROR_NOT_LAST_DEVICE => Error::NOT_LAST_DEVICE(Error::get_error_string(error_code)),
            _ => Error::OTHER(Error::get_error_string(error_code))
        }
    }
}

/// Info about each HackRF device found
#[derive(Debug)]
pub struct DeviceInfo<'a> {
    serial: &'a str,
    board_id: hackrf_usb_board_id,
}

/// A HackRF device
#[derive(Debug)]
pub struct Device {
    device: *mut hackrf_device
}

/// The hackrf library
pub struct HackRF {
    device_list: *mut hackrf_device_list_t,
    opened_devices: HashSet<*mut hackrf_device>
}

impl HackRF {
    /// Construct a new instance of the HackRF library
    pub fn new() -> Result<HackRF, Error> {
        unsafe {
            let ret = hackrf_init();  // init the library

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }

            // get and save the raw pointer to the device list
            // because we'll want to free this on drop
            let device_list : *mut hackrf_device_list_t = hackrf_device_list();

            if device_list.is_null() {
                panic!("Return from hackrf_device_list is NULL");
            }

            Ok( HackRF { device_list, opened_devices: HashSet::new() } )
        }
    }

    /// Get the list of devices found in the system
    pub fn get_device_list(&self) -> Result<Vec<DeviceInfo>, Error> {
        let mut ret = Vec::new();

        unsafe {
            info!("DEV COUNT: {}", (*self.device_list).devicecount);
            info!("USB DEV COUNT: {}", (*self.device_list).usb_devicecount);

            for i in 0..(*self.device_list).devicecount as isize {
                debug!("SERIAL: {:?}", (*self.device_list).serial_numbers.offset(i));
                debug!("BOARD ID: {:?}", *((*self.device_list).usb_board_ids.offset(i)));

                ret.push(DeviceInfo {
                    serial: CStr::from_ptr(*((*self.device_list).serial_numbers.offset(i))).to_str().expect("Error converting serial number"),
                    board_id: *((*self.device_list).usb_board_ids.offset(i))
                })
            }
        }

        debug!("RET: {:?}", ret);

        Ok(ret)
    }

    /// Open a device instance given the index into the device list
    pub fn open_device(&mut self, index: i32) -> Result<Device, Error> {
        unsafe {
            if index < 0 || index > (*self.device_list).devicecount {
                let err_str = format!("Index must be between 0 and {}", (*self.device_list).devicecount);
                return Err(Error::INVALID_PARAM(err_str));
            }

            let mut device_ptr : *mut hackrf_device = mem::uninitialized();
            let device_ptr_ptr : *mut *mut hackrf_device = &mut device_ptr;

            let ret = hackrf_device_list_open(self.device_list, index, device_ptr_ptr);

            // make sure we successfully opened the device
            if ret != hackrf_error_HACKRF_SUCCESS {
                let err = Error::from_code(ret);
                debug!("Error calling open_device: {}", err);
                return Err(err);
            }

            // add to the list of devices we've opened
            self.opened_devices.insert(device_ptr);

            debug!("Opened device: {:?}", device_ptr);

            Ok( Device { device: device_ptr } )
        }
    }
}

impl Drop for HackRF {
    fn drop(&mut self) {
        unsafe {
            // free all the opened devices
            for device_ptr in self.opened_devices.iter() {
                let ret = hackrf_close(*device_ptr);

                if ret != hackrf_error_HACKRF_SUCCESS {
                    panic!("Error calling hackrf_close({:?}): {}", device_ptr, Error::from_code(ret));
                }
            }

            // free the device list
            hackrf_device_list_free(self.device_list);

            // call exit for the library
            let ret = hackrf_exit();

            trace!("Called hackrf_exit()");

            if ret != hackrf_error_HACKRF_SUCCESS {
                panic!("Error dropping HackRF: {}", Error::from_code(ret));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_device_list() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

        let hrf = HackRF::new().expect("Error creating HackRF");

        let device_list = hrf.get_device_list().expect("Error getting device list");

        println!("Device list: {:?}", device_list);
    }

    fn open_device(index: i32) -> Result<Device, Error> {
        let mut hrf = HackRF::new().expect("Error creating HackRF");

        hrf.open_device(index)
    }

    #[test]
    fn open_device_bad_index() {
        LOGGER_INIT.call_once(|| simple_logger::init_with_level(log::Level::Trace).unwrap());

//        assert!(open_device(-1).is_err(), "Did not get error on negative index");
//        assert!(open_device(10).is_err(), "Did not get error on large index");

        println!("{:?}", open_device(0).unwrap());
    }
}
