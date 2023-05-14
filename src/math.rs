const fn compute_factorial(n: u64) -> u64 {
    if n == 0 {
        return 1;
    } else {
        return n * compute_factorial(n - 1);
    }
}

const FACTORIAL: [u64; 16] = {
    let mut result = [0; 16];
    let mut i = 0;
    while i < 16 {
        result[i] = compute_factorial(i as u64) as u64;
        i += 1;
    }
    result
};

pub const fn factorial(n: u8) -> u64 {
    FACTORIAL[n as usize]
}

pub const fn compute_n_choose_k(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    compute_factorial(n) / (compute_factorial(k) * compute_factorial(n - k))
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

pub const fn n_choose_k(n: u8, k: u8) -> u16 {
    N_CHOOSE_K[n as usize][k as usize]
}

pub const fn gcd(a: usize, b: usize) -> usize {
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

pub const fn lcm(a: usize, b: usize) -> usize {
    a * b / gcd(a, b)
}
