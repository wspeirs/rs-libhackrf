#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use]
extern crate log;

use std::fmt;
use std::ffi::CStr;

///
/// Conversion of enum hackrf_error that includes the result of a call to hackrf_error_name
///
#[derive(Debug)]
pub enum Error {
    SUCCESS,
    TRUE,
    INVALID_PARAM(&'static str),
    NOT_FOUND(&'static str),
    BUSY(&'static str),
    NO_MEMORY(&'static str),
    LIBUSB(&'static str),
    THREAD(&'static str),
    STREAMING_THREAD_ERR(&'static str),
    STREAMING_STOPPED(&'static str),
    STREAMING_EXIT_CALLED(&'static str),
    USB_API_VERSION(&'static str),
    NOT_LAST_DEVICE(&'static str),
    OTHER(&'static str)
}

impl fmt::Display for Error {
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

impl Error {
    fn get_error_string(error_code: i32) -> &'static str {
        unsafe {
            let char_ptr = hackrf_error_name(error_code);
            let c_str = CStr::from_ptr(char_ptr);

            return c_str.to_str().expect("Error converting hackrf_error_name");
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

///
/// hackrf library
pub struct HackRF {
    device_list: *mut hackrf_device_list_t
}

impl HackRF {
    pub fn new() -> Result<HackRF, Error> {
        simple_logger::init_with_level(log::Level::Trace).unwrap();

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

            Ok( HackRF { device_list } )
        }
    }

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
}

impl Drop for HackRF {
    fn drop(&mut self) {
        unsafe {
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
    fn it_works() {
        let hrf = HackRF::new().expect("Error creating HackRF");

        hrf.get_device_list();
    }
}
