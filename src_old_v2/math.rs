const FACTORIAL: [u64; 16] = [
    1,
    1,
    2,
    6,
    24,
    120,
    720,
    5040,
    40320,
    362880,
    3628800,
    39916800,
    479001600,
    6227020800,
    87178291200,
    1307674368000,
];

pub const fn factorial(n: u8) -> u64 {
    FACTORIAL[n as usize]
}

pub const fn n_choose_k(n: u8, k: u8) -> u16 {
    (factorial(n) / (factorial(k) * factorial(n - k))) as u16
}
