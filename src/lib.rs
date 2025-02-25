//! [![CI Status]][workflow] [![MSRV]][repo] [![Latest Version]][crates.io]
//! [![Rust Doc Crate]][docs.rs] [![Rust Doc Main]][docs]
//!
//! [CI Status]: https://img.shields.io/github/actions/workflow/status/juntyr/tthresh-rs/ci.yml?branch=main
//! [workflow]: https://github.com/juntyr/tthresh-rs/actions/workflows/ci.yml?query=branch%3Amain
//!
//! [MSRV]: https://img.shields.io/badge/MSRV-1.82.0-blue
//! [repo]: https://github.com/juntyr/tthresh-rs
//!
//! [Latest Version]: https://img.shields.io/crates/v/tthresh
//! [crates.io]: https://crates.io/crates/tthresh
//!
//! [Rust Doc Crate]: https://img.shields.io/docsrs/tthresh
//! [docs.rs]: https://docs.rs/tthresh/
//!
//! [Rust Doc Main]: https://img.shields.io/badge/docs-main-blue
//! [docs]: https://juntyr.github.io/tthresh-rs/tthresh
//!
//! High-level bindigs to the [tthresh] compressor.
//!
//! [tthresh]: https://github.com/rballester/tthresh

#[derive(Clone, Copy, Debug, PartialEq)]
/// Error bound
pub enum ErrorBound {
    /// Relative error
    Eps(f64),
    /// Root mean square error
    RMSE(f64),
    /// Peak signal-to-noise ratio
    PSNR(f64),
}

/// Buffer for typed decompressed data
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[allow(missing_docs)]
pub enum Buffer {
    U8(Vec<u8>),
    U16(Vec<u16>),
    I32(Vec<i32>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}

/// Compress the `data` buffer using the `target` error bound.
///
/// # Errors
///
/// Errors with
/// - [`Error::InsufficientDimensionality`] if the `data`'s `shape` is not at least
///   three-dimensional
/// - [`Error::InvalidShape`] if the `shape` does not match the `data` length
/// - [`Error::ExcessiveSize`] if the shape cannot be converted into [`u32`]s
/// - [`Error::NegativeErrorBound`] if the `target` error bound is negative
pub fn compress<T: Element>(
    data: &[T],
    shape: &[usize],
    target: ErrorBound,
    verbose: bool,
    debug: bool,
) -> Result<Vec<u8>, Error> {
    if shape.len() < 3 {
        return Err(Error::InsufficientDimensionality);
    }

    if shape.iter().copied().product::<usize>() != data.len() {
        return Err(Error::InvalidShape);
    }

    let shape = shape
        .iter()
        .copied()
        .map(u32::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Error::ExcessiveSize)?;

    let target_value = match target {
        ErrorBound::Eps(v) | ErrorBound::RMSE(v) | ErrorBound::PSNR(v) => v,
    };

    if target_value < 0.0 {
        return Err(Error::NegativeErrorBound);
    }

    let mut output = std::ptr::null_mut();
    let mut output_size = 0;

    #[allow(unsafe_code)] // FFI
    unsafe {
        tthresh_sys::compress_buffer(
            data.as_ptr().cast::<std::ffi::c_char>(),
            T::IO_TYPE,
            shape.as_ptr(),
            shape.len(),
            std::ptr::from_mut(&mut output),
            std::ptr::from_mut(&mut output_size),
            match target {
                ErrorBound::Eps(_) => tthresh_sys::Target_eps,
                ErrorBound::RMSE(_) => tthresh_sys::Target_rmse,
                ErrorBound::PSNR(_) => tthresh_sys::Target_psnr,
            },
            target_value,
            Some(alloc),
            verbose,
            debug,
        );
    }

    #[allow(unsafe_code)]
    // Safety: the output was allocated in Rust using `alloc` with the correct
    //         size and alignment
    let compressed = unsafe { Vec::from_raw_parts(output, output_size, output_size) };

    Ok(compressed)
}

/// Deompress the `compressed` bytes into a [`Buffer`] and shape.
///
/// # Errors
///
/// Errors with
/// - [`Error::ExcessiveSize`] if the output shape cannot be converted from [`u32`]s
/// - [`Error::CorruptedCompressedBytes`] if the `compressed` bytes are corrupted
pub fn decompress(
    compressed: &[u8],
    verbose: bool,
    debug: bool,
) -> Result<(Buffer, Vec<usize>), Error> {
    let mut shape = std::ptr::null_mut();
    let mut shape_size = 0;

    let mut output = std::ptr::null_mut();
    let mut output_type = 0;
    let mut output_length = 0;

    #[allow(unsafe_code)] // FFI
    let ok = unsafe {
        tthresh_sys::decompress_buffer(
            compressed.as_ptr(),
            compressed.len(),
            std::ptr::from_mut(&mut output),
            std::ptr::from_mut(&mut output_type),
            std::ptr::from_mut(&mut output_length),
            std::ptr::from_mut(&mut shape),
            std::ptr::from_mut(&mut shape_size),
            Some(alloc),
            verbose,
            debug,
        )
    };

    if !ok {
        return Err(Error::CorruptedCompressedBytes);
    }

    #[allow(unsafe_code)]
    // Safety: the shape was allocated in Rust using `alloc` with the correct
    //         size and alignment
    let shape = unsafe { Vec::from_raw_parts(shape, shape_size, shape_size) };

    #[allow(unsafe_code)]
    // Safety: the output was allocated in Rust using `alloc` with the correct
    //         size and alignment
    let decompressed = match output_type {
        tthresh_sys::IOType_uchar_ => {
            Buffer::U8(unsafe { Vec::from_raw_parts(output.cast(), output_length, output_length) })
        }
        tthresh_sys::IOType_ushort_ => {
            Buffer::U16(unsafe { Vec::from_raw_parts(output.cast(), output_length, output_length) })
        }
        tthresh_sys::IOType_int_ => {
            Buffer::I32(unsafe { Vec::from_raw_parts(output.cast(), output_length, output_length) })
        }
        tthresh_sys::IOType_float_ => {
            Buffer::F32(unsafe { Vec::from_raw_parts(output.cast(), output_length, output_length) })
        }
        tthresh_sys::IOType_double_ => {
            Buffer::F64(unsafe { Vec::from_raw_parts(output.cast(), output_length, output_length) })
        }
        #[allow(clippy::unreachable)]
        _ => unreachable!("tthresh decompression returned an unknown output type"),
    };

    let shape = shape
        .into_iter()
        .map(usize::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Error::ExcessiveSize)?;

    Ok((decompressed, shape))
}

#[derive(Debug, thiserror::Error)]
/// Errors that can occur during compression and decompression with tthresh
pub enum Error {
    /// data must be at least three-dimensional
    #[error("data must be at least three-dimensional")]
    InsufficientDimensionality,
    /// shape does not match the provided buffer
    #[error("shape does not match the provided buffer")]
    InvalidShape,
    /// data shape sizes must fit within [0; 2^32 - 1]
    #[error("data shape sizes must fit within [0; 2^32 - 1]")]
    ExcessiveSize,
    /// error bound must be non-negative
    #[error("error bound must be non-negative")]
    NegativeErrorBound,
    /// compressed bytes have been corrupted
    #[error("compressed bytes have been corrupted")]
    CorruptedCompressedBytes,
}

/// Marker trait for element types that can be compressed with tthresh
pub trait Element: sealed::Element {}

mod sealed {
    pub trait Element: Copy {
        const IO_TYPE: tthresh_sys::IOType;
    }
}

impl Element for u8 {}
impl sealed::Element for u8 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_uchar_;
}

impl Element for u16 {}
impl sealed::Element for u16 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_ushort_;
}

impl Element for i32 {}
impl sealed::Element for i32 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_int_;
}

impl Element for f32 {}
impl sealed::Element for f32 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_float_;
}

impl Element for f64 {}
impl sealed::Element for f64 {
    const IO_TYPE: tthresh_sys::IOType = tthresh_sys::IOType_double_;
}

extern "C" fn alloc(size: usize, align: usize) -> *mut std::ffi::c_void {
    #[allow(clippy::unwrap_used)]
    let layout = std::alloc::Layout::from_size_align(size, align).unwrap();

    // return a dangling pointer if the layout is zero-sized
    if layout.size() == 0 {
        #[allow(clippy::useless_transmute)]
        // FIXME: use std::ptr::without_provenance_mut with MSRV 1.84
        #[allow(unsafe_code)]
        // Safety: usize -> *mut is always safe
        return unsafe { std::mem::transmute(align) };
    }

    #[allow(unsafe_code)]
    // Safety: layout is not zero-sized
    unsafe { std::alloc::alloc_zeroed(layout) }.cast()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn compress_decompress(target: ErrorBound) {
        let data = std::fs::read("tthresh-sys/tthresh/data/3D_sphere_64_uchar.raw")
            .expect("input file should not be missing");

        let compressed = compress(data.as_slice(), &[64, 64, 64], target, true, true)
            .expect("compression should not fail");

        let (decompressed, shape) =
            decompress(compressed.as_slice(), true, true).expect("decompression should not fail");
        assert!(matches!(decompressed, Buffer::U8(_)));
        assert_eq!(shape, &[64, 64, 64]);
    }

    #[test]
    fn compress_decompress_eps() {
        compress_decompress(ErrorBound::Eps(0.5));
    }

    #[test]
    fn compress_decompress_rmse() {
        compress_decompress(ErrorBound::RMSE(0.1));
    }

    #[test]
    fn compress_decompress_psnr() {
        compress_decompress(ErrorBound::PSNR(30.0));
    }

    #[test]
    fn compress_decompress_u8() {
        let compressed = compress(&[42_u8], &[1, 1, 1], ErrorBound::RMSE(0.0), true, true)
            .expect("compression should not fail");

        let (decompressed, shape) =
            decompress(compressed.as_slice(), true, true).expect("decompression should not fail");
        assert_eq!(decompressed, Buffer::U8(vec![42]));
        assert_eq!(shape, &[1, 1, 1]);
    }

    #[test]
    fn compress_decompress_u16() {
        let compressed = compress(&[42_u16], &[1, 1, 1], ErrorBound::RMSE(0.0), true, true)
            .expect("compression should not fail");

        let (decompressed, shape) =
            decompress(compressed.as_slice(), true, true).expect("decompression should not fail");
        assert_eq!(decompressed, Buffer::U16(vec![42]));
        assert_eq!(shape, &[1, 1, 1]);
    }

    #[test]
    fn compress_decompress_i32() {
        let compressed = compress(&[42_i32], &[1, 1, 1], ErrorBound::RMSE(0.0), true, true)
            .expect("compression should not fail");

        let (decompressed, shape) =
            decompress(compressed.as_slice(), true, true).expect("decompression should not fail");
        assert_eq!(decompressed, Buffer::I32(vec![42]));
        assert_eq!(shape, &[1, 1, 1]);
    }

    #[test]
    fn compress_decompress_f32() {
        let compressed = compress(&[42.0_f32], &[1, 1, 1], ErrorBound::RMSE(0.0), true, true)
            .expect("compression should not fail");

        let (decompressed, shape) =
            decompress(compressed.as_slice(), true, true).expect("decompression should not fail");
        assert_eq!(decompressed, Buffer::F32(vec![42.0]));
        assert_eq!(shape, &[1, 1, 1]);
    }

    #[test]
    fn compress_decompress_f64() {
        let compressed = compress(&[42.0_f64], &[1, 1, 1], ErrorBound::RMSE(0.0), true, true)
            .expect("compression should not fail");

        let (decompressed, shape) =
            decompress(compressed.as_slice(), true, true).expect("decompression should not fail");
        assert_eq!(decompressed, Buffer::F64(vec![42.0]));
        assert_eq!(shape, &[1, 1, 1]);
    }

    #[test]
    fn decompress_empty_garbage() {
        let result = decompress(&[0], true, true);
        assert!(matches!(result, Err(Error::CorruptedCompressedBytes)));
    }

    #[test]
    fn decompress_full_garbage() {
        let result = decompress(vec![1; 1024].as_slice(), true, true);
        assert!(matches!(result, Err(Error::CorruptedCompressedBytes)));
    }
}
