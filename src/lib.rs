#![allow(missing_docs)] // FIXME

use std::ffi::{CString, NulError};

use tthresh_sys::my_main;

pub fn tthresh(argv: &[&str]) {
    let mut argv = argv.iter().copied().map(|a| Ok(CString::new(a)?.into_raw())).collect::<Result<Vec<_>, NulError>>().unwrap();

    #[allow(unsafe_code)]
    let result = unsafe {
        my_main(argv.len().try_into().unwrap(), argv.as_mut_ptr())
    };

    assert_eq!(result, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help() {
        tthresh(&["tthresh", "-h"]);
    }

    #[test]
    fn compress() {
        tthresh(&["tthresh", "-i", "tthresh-sys/tthresh/data/3D_sphere_64_uchar.raw", "-t", "uchar", "-s", "64", "64", "64", "-p", "30", "-c", "3D_sphere_64_uchar.compressed", "-v", "-d"]);
    }
}
