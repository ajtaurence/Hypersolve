fn cycle_length(permutation: &[u8], start: u8, visited: &mut [bool]) -> u8 {
    let mut count = 1;
    visited[start as usize] = true;
    let mut index = permutation[start as usize];
    visited[index as usize] = true;

    while index != start {
        index = permutation[index as usize];
        visited[index as usize] = true;
        count += 1;
    }

    count
}

pub fn is_odd<const N: usize>(permutation: [u8; N]) -> bool {
    let length = permutation.len();
    let mut visited = [false; N];

    let mut is_odd = false;

    for i in 0..length {
        if !visited[i] {
            is_odd ^= cycle_length(&permutation, i as u8, &mut visited) % 2 == 0;
        }
    }

    is_odd
}
