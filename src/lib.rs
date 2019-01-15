#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use]
extern crate log;


#[cfg(test)] use std::sync::{Once, ONCE_INIT};
#[cfg(test)] static LOGGER_INIT: Once = ONCE_INIT;


pub mod error;
pub mod hackrf;
pub mod device;

