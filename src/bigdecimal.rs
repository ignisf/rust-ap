#![crate_id = "rust-arbitrary-precision#0.1"]

#![desc = "GMP/MPFR-backed Big{Integer, Rational, Float}"]
#![license = "BSD"]

#![crate_type = "lib"]
#![allow(non_camel_case_types)]

mod ap {
    use std::libc::{c_char, c_int, c_long, c_void};
    use std::mem::uninit;
    use std::num::FromStrRadix;

    type mpfr_prec_t = c_long;
    type mpfr_exp_t = c_long;
    type mpfr_sign_t = c_int;
    type mpfr_rnd_t = c_int;

    type mpfr_srcptr = *mpfr_struct;
    type mpfr_ptr = *mut mpfr_struct;

    struct mpfr_struct {
        _mpfr_prec: mpfr_prec_t,
        _mpfr_sign: mpfr_sign_t,
        _mpfr_exp: mpfr_exp_t,
        _mpfr_d: *c_void
    }

    #[link(name = "mpfr")]
    extern {
        fn mpfr_clear(rop: mpfr_ptr);
        fn mpfr_init_set_str(x: mpfr_ptr, s: *c_char, base: c_int, rnd: mpfr_rnd_t) -> c_int;
    }

    pub struct BigDecimal {
        priv mpfr: mpfr_struct,
    }

    impl BigDecimal {}

    impl Drop for BigDecimal {
        fn drop(&mut self) {
            unsafe { mpfr_clear(&mut self.mpfr) }
        }
    }

    impl FromStrRadix for BigDecimal {

        /**
         * Create a new BigDecimal from a string and a specified radix.
         */
        fn from_str_radix(str: &str, radix: uint) -> Option<BigDecimal> {
            assert!(radix == 0 || (radix >= 2 && radix <= 62));
            unsafe {
                let mut mpfr: mpfr_struct = uninit();
                let r = str.with_c_str(|s| mpfr_init_set_str(&mut mpfr, s, radix as i32, 0));
                if r == 0 {
                    Some(BigDecimal { mpfr: mpfr })
                } else {
                    mpfr_clear(&mut mpfr);
                    None
                }
            }
        }
    }

    #[cfg(test)]
    mod bigdecimal_tests {
        use super::BigDecimal;
        use std::num::FromStrRadix;

        #[test]
        fn test_from_str_radix() {
            let bigdecimal: Option<BigDecimal> = FromStrRadix::from_str_radix("2", 10);
        }
    }
}
