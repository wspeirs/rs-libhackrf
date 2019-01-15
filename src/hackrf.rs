
use std::collections::HashSet;
use std::ffi::CStr;
use std::mem;

use crate::device::Device;
use crate::error::Error;

use crate::{
    hackrf_device_list_t,
    hackrf_device,
    hackrf_usb_board_id,
    hackrf_error_HACKRF_SUCCESS,
    hackrf_init,
    hackrf_exit,
    hackrf_device_list,
    hackrf_device_list_free,
    hackrf_device_list_open,
    hackrf_close
};

/// The hackrf library
pub struct HackRF {
    device_list: *mut hackrf_device_list_t,
    opened_devices: HashSet<*mut hackrf_device>
}

/// Info about each HackRF device found
#[derive(Debug)]
pub struct DeviceInfo<'a> {
    serial: &'a str,
    board_id: hackrf_usb_board_id,
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
    use crate::LOGGER_INIT;

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
