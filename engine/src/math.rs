//! Compile-time-selected math backend.
//!
//! Native runners use Rust's standard-library math, ESP-IDF firmware links
//! directly to the target toolchain's C `libm`, and other `no_std` targets use
//! the portable Rust `libm` crate. These are `cfg` branches, not runtime
//! dispatch: only one implementation is compiled for a given target.

#[cfg(target_os = "espidf")]
mod backend {
    #[link(name = "m")]
    unsafe extern "C" {
        #[link_name = "sinf"]
        fn c_sinf(value: f32) -> f32;
        #[link_name = "cosf"]
        fn c_cosf(value: f32) -> f32;
        #[link_name = "fmodf"]
        fn c_fmodf(value: f32, modulus: f32) -> f32;
        #[link_name = "powf"]
        fn c_powf(value: f32, exponent: f32) -> f32;
    }

    #[inline]
    pub fn sinf(value: f32) -> f32 {
        // SAFETY: C libm functions have no Rust-side memory or lifetime
        // requirements and ESP-IDF links the target's `libm` statically.
        unsafe { c_sinf(value) }
    }

    #[inline]
    pub fn cosf(value: f32) -> f32 {
        // SAFETY: See `sinf`.
        unsafe { c_cosf(value) }
    }

    #[inline]
    pub fn fmodf(value: f32, modulus: f32) -> f32 {
        // SAFETY: See `sinf`.
        unsafe { c_fmodf(value, modulus) }
    }

    #[inline]
    pub fn powf(value: f32, exponent: f32) -> f32 {
        // SAFETY: See `sinf`.
        unsafe { c_powf(value, exponent) }
    }
}

#[cfg(all(not(target_os = "espidf"), any(test, feature = "std")))]
mod backend {
    #[inline]
    pub fn sinf(value: f32) -> f32 {
        value.sin()
    }

    #[inline]
    pub fn cosf(value: f32) -> f32 {
        value.cos()
    }

    #[inline]
    pub fn fmodf(value: f32, modulus: f32) -> f32 {
        value % modulus
    }

    #[inline]
    pub fn powf(value: f32, exponent: f32) -> f32 {
        value.powf(exponent)
    }
}

#[cfg(all(not(target_os = "espidf"), not(any(test, feature = "std"))))]
mod backend {
    pub use libm::{cosf, fmodf, powf, sinf};
}

pub(crate) use backend::{cosf, fmodf, powf, sinf};

#[inline]
pub(crate) fn fabsf(value: f32) -> f32 {
    // `abs` is a sign-bit operation on every backend; it needs no libm call.
    value.abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_backend_matches_reference_vectors() {
        for step in -1_000..=1_000 {
            let value = step as f32 * 0.01;
            assert!((sinf(value) - libm::sinf(value)).abs() < 1e-6);
            assert!((cosf(value) - libm::cosf(value)).abs() < 1e-6);
        }
        assert_eq!(fmodf(5.5, 2.0), libm::fmodf(5.5, 2.0));
        assert!((powf(0.75, 2.4) - libm::powf(0.75, 2.4)).abs() < 1e-6);
    }
}
