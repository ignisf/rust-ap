#![crate_id = "rust-arbitrary-precision#0.1"]

#![desc = "GMP/MPFR-backed Big{Integer, Rational, Float}"]
#![license = "BSD"]

#![crate_type = "lib"]
#![allow(non_camel_case_types)]
#![feature(link_args)]

mod ap {
    use std::libc::{c_char, c_int, c_long, c_ulong, c_void, size_t};
    use std::mem::uninit;
    use std::num::{FromStrRadix, Zero};
    use std::cmp::{Eq, Ord};
    use std::ops::Add;
    use std::from_str::FromStr;
    use std::fmt::{Formatter, Show, Result};
    use std::c_str::CString;

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

    #[link_args = "-lmpfr -lgmp"]
    extern {
        fn mpfr_init(x: mpfr_ptr);
        fn mpfr_clear(x: mpfr_ptr);
        fn mpfr_init_set_str(x: mpfr_ptr, s: *c_char, base: c_int, rnd: mpfr_rnd_t) -> c_int;
        fn mpfr_set_zero(x: mpfr_ptr, sign: c_int);
        fn mpfr_equal_p(op1: mpfr_srcptr, op2: mpfr_srcptr) -> c_int;
        fn mpfr_less_p(op1: mpfr_srcptr, op2: mpfr_srcptr) -> c_int;
        fn mpfr_add(rop: mpfr_ptr, op1: mpfr_srcptr, op2: mpfr_srcptr, rnd: mpfr_rnd_t) -> c_int;
        fn mpfr_set_si(rop: mpfr_ptr, op: c_long, rnd: mpfr_rnd_t) -> c_int;
        fn mpfr_set_ui(rop: mpfr_ptr, op: c_ulong, rnd: mpfr_rnd_t) -> c_int;
        fn mpfr_get_prec(x: mpfr_srcptr) -> mpfr_prec_t;
        fn mpfr_get_str(str: *c_char, expptr: *mpfr_exp_t, b: c_int, n: size_t, op: mpfr_srcptr, rnd: mpfr_rnd_t) -> *c_char;
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

        pub fn get_precision(&self) -> u64 {
            unsafe { mpfr_get_prec(&self.mpfr) as u64 }
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

    impl Add<BigDecimal, BigDecimal> for BigDecimal {
        /**
         * Return a new BigDecimal rounded towards the nearest number that
         * consists of the sum of senf and rhs.
         */
        #[inline]
        fn add(&self, rhs: &BigDecimal) -> BigDecimal {
            unsafe {
                let mut result = BigDecimal::new();
                mpfr_add(&mut result.mpfr, &self.mpfr, &rhs.mpfr, 0);
                result
            }
        }
    }

    impl Zero for BigDecimal {
        /**
         * Returns a BigDecimal that represents 0
         */
        #[inline]
        fn zero() -> BigDecimal {
            unsafe {
                let mut result = BigDecimal::new();
                mpfr_set_zero(&mut result.mpfr, 0);
                result
            }
        }

        /**
         * Returns true if a BigDecimal is 0
         */
        #[inline]
        fn is_zero(&self) -> bool {
            let zero: BigDecimal = Zero::zero();
            self == &zero
        }
    }

    impl FromPrimitive for BigDecimal {
        /**
         * Create a new BigDecimal from a i64
         */
        #[inline]
        fn from_i64(n: i64) -> Option<BigDecimal> {
            unsafe {
                let mut result = BigDecimal::new();
                mpfr_set_si(&mut result.mpfr, n, 0);
                Some(result)
            }
        }

        /**
         * Create a new BigDecimal from a u64
         */
        #[inline]
        fn from_u64(n: u64) -> Option<BigDecimal> {
            unsafe {
                let mut result = BigDecimal::new();
                mpfr_set_ui(&mut result.mpfr, n, 0);
                Some(result)
            }
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

    impl FromStr for BigDecimal {
        /**
         * Create a BigDecimal from a string in base 10
         */
        #[inline]
        fn from_str(str: &str) -> Option<BigDecimal> {
            FromStrRadix::from_str_radix(str, 10)
        }
    }

    impl Show for BigDecimal {
        fn fmt(&self, f: &mut Formatter) -> Result {
            use std::ptr::null;
            let exp: mpfr_exp_t = 0;
            let result = unsafe {
                let res_ptr = mpfr_get_str(null(), &exp, 10, 0, &self.mpfr, 0);
                CString::new(res_ptr, true)
            };
            let string = result.as_str().unwrap();
            write!(f.buf, "{}", [string.slice_to(exp as uint), ".", string.slice_from(exp as uint)].concat())
        }
    }

    #[cfg(test)]
    mod bigdecimal_tests {
        use super::BigDecimal;
        use std::num::{FromStrRadix, Zero};
        use std::from_str::FromStr;

        #[test]
        fn check_if_zero_is_equal_to_zero() {
            let zero: BigDecimal = FromStr::from_str("0").unwrap();
            let zero_again: BigDecimal = FromStr::from_str("0").unwrap();

            assert_eq!(zero, zero_again);
        }

        #[test]
        #[should_fail]
        fn check_if_zero_is_not_equal_to_one() {
            let zero: BigDecimal = Zero::zero();
            let one: BigDecimal = FromStr::from_str("1").unwrap();

            assert_eq!(zero, one);
        }

        #[test]
        fn check_if_zero_is_less_than_one() {
            let zero: BigDecimal = Zero::zero();
            let one: BigDecimal = FromStr::from_str("1").unwrap();

            assert!(zero < one);
        }

        #[test]
        #[should_fail]
        fn check_if_zero_is_not_more_than_zero() {
            let zero: BigDecimal = Zero::zero();
            let zero_again: BigDecimal = Zero::zero();

            assert!(zero < zero_again);
        }

        #[test]
        fn test_addition() {
            let one_point_one: BigDecimal = FromStr::from_str("1.1").unwrap();
            let one_point_nine: BigDecimal = FromStr::from_str("1.9").unwrap();
            let three: BigDecimal = FromStr::from_str("3").unwrap();

            assert_eq!(one_point_one + one_point_nine, three);
        }

        #[test]
        fn test_zero() {
            let zero_from_str: BigDecimal = FromStr::from_str("0").unwrap();
            let one_point_nine: BigDecimal = FromStr::from_str("1.9").unwrap();
            let zero: BigDecimal = Zero::zero();

            assert_eq!(zero, zero_from_str);
            assert!(zero != one_point_nine);
        }

        #[test]
        fn test_is_zero() {
            let zero_from_str: BigDecimal = FromStr::from_str("0").unwrap();
            let zero: BigDecimal = Zero::zero();
            assert!(zero.is_zero());
            assert!(zero_from_str.is_zero());
        }

        #[test]
        fn test_from_str() {
            let one_point_one_from_str_radix: BigDecimal = FromStrRadix::from_str_radix("1.1", 10).unwrap();
            let one_point_one_from_str: BigDecimal = FromStr::from_str("1.1").unwrap();
            assert_eq!(one_point_one_from_str, one_point_one_from_str_radix);
        }

        #[test]
        fn test_from_i64() {
            let one_from_str: BigDecimal = FromStr::from_str("1").unwrap();
            let one_from_i64: BigDecimal = FromPrimitive::from_i64(1 as i64).unwrap();
            assert_eq!(one_from_i64, one_from_str);
        }

        #[test]
        fn test_from_u64() {
            let one_from_str: BigDecimal = FromStr::from_str("1").unwrap();
            let one_from_u64: BigDecimal = FromPrimitive::from_u64(1 as u64).unwrap();
            assert_eq!(one_from_u64, one_from_str);
        }

        #[test]
        fn test_from_f64() {
            let one_from_str: BigDecimal = FromStr::from_str("111.8").unwrap();
            let one_from_f64: BigDecimal = FromPrimitive::from_f64(1.2f64).unwrap();
            assert_eq!(one_from_f64, one_from_str);
        }
    }
}
