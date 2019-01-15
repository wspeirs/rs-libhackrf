use std::fmt;
use std::ffi::CStr;

use crate::{
    hackrf_error_HACKRF_SUCCESS,
    hackrf_error_HACKRF_TRUE,
    hackrf_error_HACKRF_ERROR_INVALID_PARAM,
    hackrf_error_HACKRF_ERROR_NOT_FOUND,
    hackrf_error_HACKRF_ERROR_BUSY,
    hackrf_error_HACKRF_ERROR_NO_MEM,
    hackrf_error_HACKRF_ERROR_LIBUSB,
    hackrf_error_HACKRF_ERROR_THREAD,
    hackrf_error_HACKRF_ERROR_STREAMING_THREAD_ERR,
    hackrf_error_HACKRF_ERROR_STREAMING_STOPPED,
    hackrf_error_HACKRF_ERROR_STREAMING_EXIT_CALLED,
    hackrf_error_HACKRF_ERROR_USB_API_VERSION,
    hackrf_error_HACKRF_ERROR_NOT_LAST_DEVICE,

};


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
            let char_ptr = crate::hackrf_error_name(error_code);
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
