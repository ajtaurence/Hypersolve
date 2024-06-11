/// Computes factorials
pub(crate) const fn compute_factorial(n: u64) -> u64 {
    let mut result = 1;

    let mut i = 2;
    while i <= n {
        result *= i;
        i += 1;
    }

    unsafe { assert_unchecked!(result > 0) };

    result
}

/// Returns factorials up to 15! from a lookup table
///
/// # Panics
/// Will panic if `n > 15`
pub(crate) const fn factorial(n: u8) -> u64 {
    const FACTORIAL: [u64; 16] = {
        let mut result = [0; 16];
        let mut i = 0;
        while i < 16 {
            result[i] = compute_factorial(i as u64);
            i += 1;
        }
        result
    };

    let result = FACTORIAL[n as usize];

    unsafe { assert_unchecked!(result > 0) };

    result
}

/// Computes the binomial coefficient <sub>n</sub>C<sub>k</sub> by computing all factorials
const fn compute_n_choose_k(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }

    compute_factorial(n) / (compute_factorial(k) * compute_factorial(n - k))
}

/// Returns binomial coefficients up to 8 choose 15
///
/// # Panics
/// Will panic if `n > 8` or `k > 15`
pub(crate) const fn n_choose_k(n: u8, k: u8) -> u16 {
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

    N_CHOOSE_K[n as usize][k as usize]
}
