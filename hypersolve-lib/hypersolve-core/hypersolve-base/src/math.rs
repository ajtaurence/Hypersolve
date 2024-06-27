use const_for::const_for;

use crate::assert_unchecked;

/// Computes factorials with a loop
pub const fn factorial(n: u64) -> u64 {
    let mut prod = 1;

    const_for!(i in 2..n+1 => {
        prod *= i;
    });

    // SAFTEY: factorials are always greater than zero
    unsafe { assert_unchecked!(prod > 0) };

    prod
}

/// Returns binomial coefficients up to 8 choose 15
///
/// # Safety
/// It is undefined behavior if `n > 15` or `k > 8`
pub const unsafe fn n_choose_k(n: u8, k: u8) -> u16 {
    /// Computes the binomial coefficient <sub>n</sub>C<sub>k</sub> by computing all factorials
    const fn compute_n_choose_k(mut n: u64, mut k: u64) -> u64 {
        if k > n {
            return 0;
        }

        if k > n - k {
            k = n - k;
        }

        let mut c = 1;
        let mut i = 1;
        while i <= k {
            // SAFTEY: i is only incremented from 1
            unsafe { assert_unchecked!(i > 0) };

            c = c * n / i;

            i += 1;
            n -= 1;
        }

        c
    }

    const N_CHOOSE_K: [[u16; 9]; 16] = {
        let mut result = [[0; 9]; 16];
        let mut n = 0;
        while n < 16 {
            let mut k = 0;
            while k < 9 {
                result[n][k] = compute_n_choose_k(n as u64, k as u64) as u16;
                k += 1;
            }
            n += 1;
        }
        result
    };

    assert_unchecked!(n < 16 && k < 9);

    N_CHOOSE_K[n as usize][k as usize]
}

/// computes the greatest common denomonator of `a` and `b` using Euclid's algorithm
const fn gcd(a: usize, b: usize) -> usize {
    let mut max = a;
    let mut min = b;
    if min > max {
        let val = max;
        max = min;
        min = val;
    }
    loop {
        let res = max % min;
        if res == 0 {
            return min;
        }
        max = min;
        min = res;
    }
}

/// Computes the least common multiple of `a` and `b` by computing their gcd
pub const fn lcm(a: usize, b: usize) -> usize {
    a * b / gcd(a, b)
}
