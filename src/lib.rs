#![allow(missing_docs)] // FIXME
#![allow(clippy::unwrap_used)] // FIXME
#![allow(clippy::missing_panics_doc)] // FIXME

use std::{ffi::CString, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IoType {
    U8,
    U16,
    I32,
    F32,
    F64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    Eps(f64),
    RMSE(f64),
    PSNR(f64),
}

pub fn compress(
    shape: &[u32],
    input_file: &Path,
    compressed_file: &Path,
    io_type: IoType,
    target: Target,
    verbose: bool,
    debug: bool,
) {
    let input_file = CString::new(input_file.as_os_str().as_encoded_bytes()).unwrap();
    let compressed_file = CString::new(compressed_file.as_os_str().as_encoded_bytes()).unwrap();

    #[allow(unsafe_code)]
    unsafe {
        tthresh_sys::my_compress(
            shape.as_ptr(),
            shape.len(),
            input_file.as_ptr(),
            compressed_file.as_ptr(),
            match io_type {
                IoType::U8 => c"uchar",
                IoType::U16 => c"ushort",
                IoType::I32 => c"int",
                IoType::F32 => c"float",
                IoType::F64 => c"double",
            }
            .as_ptr(),
            match target {
                Target::Eps(_) => tthresh_sys::Target_eps,
                Target::RMSE(_) => tthresh_sys::Target_rmse,
                Target::PSNR(_) => tthresh_sys::Target_psnr,
            },
            match target {
                Target::Eps(v) | Target::RMSE(v) | Target::PSNR(v) => v,
            },
            verbose,
            debug,
        );
    }
}

pub fn decompress(
    shape: &[u32],
    compressed_file: &Path,
    decompressed_file: &Path,
    verbose: bool,
    debug: bool,
) {
    let compressed_file = CString::new(compressed_file.as_os_str().as_encoded_bytes()).unwrap();
    let decompressed_file = CString::new(decompressed_file.as_os_str().as_encoded_bytes()).unwrap();

    #[allow(unsafe_code)]
    unsafe {
        tthresh_sys::my_decompress(
            shape.as_ptr(),
            shape.len(),
            compressed_file.as_ptr(),
            decompressed_file.as_ptr(),
            verbose,
            debug,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_decompress() {
        compress(
            &[64, 64, 64],
            Path::new("tthresh-sys/tthresh/data/3D_sphere_64_uchar.raw"),
            Path::new("3D_sphere_64_uchar.compressed"),
            IoType::U8,
            Target::PSNR(30.0),
            true,
            true,
        );
        decompress(
            &[64, 64, 64],
            Path::new("3D_sphere_64_uchar.compressed"),
            Path::new("3D_sphere_64_uchar.decompressed"),
            true,
            true,
        );
    }
}
