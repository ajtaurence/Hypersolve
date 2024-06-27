mod a4;
mod c3;
mod k4;
mod permutation;

pub use a4::*;
pub use c3::*;
pub use k4::*;
pub use permutation::*;

pub const fn a4_k4_group_mul(a: A4, b: K4) -> K4 {
    use const_for::const_for;
    use strum::VariantArray;

    const A4_K4_MUL_TABLE: [[K4; 4]; 12] = {
        let mut result = [[K4::E; 4]; 12];

        const_for!(i in 0..12 => {
            const_for!(j in 0..12 => {
                let elem1 = A4::VARIANTS[i];
                let elem2 = A4::VARIANTS[j];

                let result_elem = A4::group_mul(elem1, elem2).to_k4();

                // make sure when we overwrite a previously calculated value that it is the same
                let existing_value = result[elem1 as usize][elem2.to_k4() as usize];
                assert!(
                    existing_value as usize == result_elem as usize
                        || existing_value as usize == K4::E as usize
                );

                result[elem1 as usize][elem2.to_k4() as usize] = result_elem;
            });
        });

        result
    };

    A4_K4_MUL_TABLE[a as usize][b as usize]
}

pub const fn c3_a4_group_mul(a: C3, b: A4) -> C3 {
    C3::group_mul(a, b.to_c3())
}
