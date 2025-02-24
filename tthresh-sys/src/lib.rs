#![allow(missing_docs)] // FIXME
#![allow(unsafe_code)] // sys-crate
#![allow(non_upper_case_globals)] // bindgen

#[cfg(feature = "openmp")]
use ::openmp_sys as _;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
