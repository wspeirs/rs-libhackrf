#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CStr;

///
/// Conversion of enum hackrf_error that includes the result of a call to hackrf_error_name
///
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

///
/// hackrf library
pub struct HackRF {

}

impl HackRF {
    pub fn new() -> Result<(), Error> {
        unsafe {
            let ret = hackrf_init();

            if ret != hackrf_error_HACKRF_SUCCESS {
                return Err(Error::from_code(ret));
            }

            Ok( () )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works()
    {
        unsafe {
            let ret = hackrf_init();

            if ret != hackrf_error_HACKRF_SUCCESS {
                let err = hackrf_error_name(ret);

                println!("{:?}", err);
            }

            let list = hackrf_device_list();

            println!("LIST: {:?}", *list);
        }
    }
}
