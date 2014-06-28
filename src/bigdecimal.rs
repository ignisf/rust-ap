#![crate_id = "rust-arbitrary-precision#0.1"]

#![desc = "GMP/MPFR-backed Big{Integer, Rational, Float}"]
#![license = "BSD"]

#![crate_type = "lib"]
#![allow(non_camel_case_types)]

mod ap {
    use std::libc::{c_char, c_int, c_long, c_void};
    use std::mem::uninit;
    use std::num::{FromStrRadix, One, Zero};
    use std::cmp::Eq;

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
        fn mpfr_init(x: mpfr_ptr);
        fn mpfr_clear(x: mpfr_ptr);
        fn mpfr_init_set_str(x: mpfr_ptr, s: *c_char, base: c_int, rnd: mpfr_rnd_t) -> c_int;
        fn mpfr_set_zero(x: mpfr_ptr, sign: c_int);
        fn mpfr_equal_p(op1: mpfr_srcptr, op2: mpfr_srcptr) -> c_int;
        fn mpfr_less_p(op1: mpfr_srcptr, op2: mpfr_srcptr) -> c_int;
    }

    pub struct BigDecimal {
        priv mpfr: mpfr_struct,
    }

    impl BigDecimal {
        pub fn new() -> BigDecimal {
            unsafe {
                let mut mpfr: mpfr_struct = uninit();
                mpfr_init(&mut mpfr);
                BigDecimal { mpfr: mpfr }
            }
        }
    }

    impl Drop for BigDecimal {
        fn drop(&mut self) {
            unsafe { mpfr_clear(&mut self.mpfr) }
        }
    }

    impl Eq for BigDecimal {
        /**
         * Return true if self = other, and false otherwise. This function
         * returns false whenever self and/or other is NaN.
         */
        #[inline]
        fn eq(&self, other: &BigDecimal) -> bool {
            unsafe { mpfr_equal_p(&self.mpfr, &other.mpfr) == 1 }
        }
    }

    impl Ord for BigDecimal {
        /**
         * Return true if self < other, and false otherwise. This function
         * returns false whenever self and/or other is NaN.
         */
        #[inline]
        fn lt(&self, other: &BigDecimal) -> bool {
            unsafe { mpfr_less_p(&self.mpfr, &other.mpfr) == 1 }
        }
    }

    impl FromStrRadix for BigDecimal {
        /**
         * Create a new BigDecimal from a string and a specified radix.
         */
        #[inline]
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
        fn check_if_zero_is_equal_to_zero() {
            let zero: BigDecimal = FromStrRadix::from_str_radix("0", 10).unwrap();
            let zero_again: BigDecimal = FromStrRadix::from_str_radix("0", 10).unwrap();

            assert!(zero == zero_again);
        }

        #[test]
        #[should_fail]
        fn check_if_zero_is_not_equal_to_one() {
            let zero: BigDecimal = FromStrRadix::from_str_radix("0", 10).unwrap();
            let one: BigDecimal = FromStrRadix::from_str_radix("1", 10).unwrap();

            assert!(zero == one);
        }

        #[test]
        fn check_if_zero_is_less_than_one() {
            let zero: BigDecimal = FromStrRadix::from_str_radix("0", 10).unwrap();
            let one: BigDecimal = FromStrRadix::from_str_radix("1", 10).unwrap();

            assert!(zero < one);
        }

        #[test]
        #[should_fail]
        fn check_if_zero_is_not_more_than_zero() {
            let zero: BigDecimal = FromStrRadix::from_str_radix("0", 10).unwrap();
            let zero_again: BigDecimal = FromStrRadix::from_str_radix("0", 10).unwrap();
            assert!(zero < zero_again);
        }
    }
}
