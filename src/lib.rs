#![allow(missing_docs)] // FIXME
#![allow(clippy::unwrap_used)] // FIXME
#![allow(clippy::missing_panics_doc)] // FIXME

use std::{
    ffi::{c_char, CString},
    path::Path,
};

pub trait Elem: sealed::Elem {}

mod sealed {
    pub trait Elem {
        const IO_TYPE: tthresh_sys::IOType;
    }
}

impl Elem for u8 {}
impl sealed::Elem for u8 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_uchar_;
}

impl Elem for u16 {}
impl sealed::Elem for u16 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_ushort_;
}

impl Elem for i32 {}
impl sealed::Elem for i32 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_int_;
}

impl Elem for f32 {}
impl sealed::Elem for f32 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_float_;
}

impl Elem for f64 {}
impl sealed::Elem for f64 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_double_;
}

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

pub fn compress<T: Elem>(
    data: &[T],
    shape: &[usize],
    target: Target,
    verbose: bool,
    debug: bool,
) -> Vec<u8> {
    assert!(shape.len() >= 3);
    assert_eq!(shape.iter().copied().product::<usize>(), data.len());
    let shape = shape
        .iter()
        .copied()
        .map(u32::try_from)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut output = std::ptr::null_mut();
    let mut noutput = 0;

    #[allow(unsafe_code)]
    unsafe {
        tthresh_sys::my_compress(
            shape.as_ptr(),
            shape.len(),
            data.as_ptr().cast::<c_char>(),
            T::IO_TYPE,
            std::ptr::from_mut(&mut output),
            std::ptr::from_mut(&mut noutput),
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

    let mut compressed = vec![0_u8; noutput];

    #[allow(unsafe_code)]
    unsafe {
        std::ptr::copy_nonoverlapping(output.cast::<u8>(), compressed.as_mut_ptr(), noutput);
    }

    #[allow(unsafe_code)]
    unsafe {
        tthresh_sys::dealloc_bytes(output);
    }

    compressed
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
        let data = std::fs::read("tthresh-sys/tthresh/data/3D_sphere_64_uchar.raw")
            .expect("missing input file");
        let compressed = compress(
            data.as_slice(),
            &[64, 64, 64],
            Target::PSNR(30.0),
            true,
            true,
        );
        std::fs::write("3D_sphere_64_uchar.compressed", compressed)
            .expect("writing decompressed data should not fail");

        decompress(
            &[64, 64, 64],
            Path::new("3D_sphere_64_uchar.compressed"),
            Path::new("3D_sphere_64_uchar.decompressed"),
            true,
            true,
        );
    }
}
