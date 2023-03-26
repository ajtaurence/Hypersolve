/* Static Data Example
const_data!(
    pub TEST_DATA: PiecePuzzle = complex_runtime_calculation()
);
*/

/// Generates lazy static data at runtime and saves to a file
#[cfg(feature = "gen-const-data")]
macro_rules! const_data {
    (pub $name:ident: $type:ty = $expr:expr) => {
        lazy_static::lazy_static! {
            pub static ref $name: $type = {
                union Transmute {
                    bytes: [u8; {std::mem::size_of::<$type>()}],
                    obj: $type
                }

                use std::io::Write;

                let data = $expr;
                let mut file = std::fs::File::create(concat!("static_data/", concat!(stringify!($name), ".dat"))).expect("unable to write static data");
                file.write_all(& unsafe { Transmute { obj: $expr }.bytes }).expect("unable to write static data");
                data
            };
        }
    };
    ($name:ident: $type:ty = $expr:expr) => {
        lazy_static::lazy_static! {
            static ref $name: $type = {
                union Transmute {
                    bytes: [u8; {std::mem::size_of::<$type>()}],
                    obj: $type
                }

                use std::io::Write;

                let data = $expr;
                let mut file = std::fs::File::create(concat!("static_data/", concat!(stringify!($name), ".dat"))).expect("unable to write static data");
                file.write_all(& unsafe { Transmute { obj: $expr }.bytes }).expect("unable to write static data");
                data
            };
        }
    };
}

/// Loads const data at compiletime from lazy static data generated at runtime
#[cfg(not(feature = "gen-const-data"))]
macro_rules! const_data {
    (pub $name:ident: $type:ty = $epr:expr) => {
        pub const $name: $type = unsafe {
            union Transmute {
                bytes: [u8; { std::mem::size_of::<$type>() }],
                obj: $type,
            }
            Transmute {
                bytes: *include_bytes!(concat!("../static_data/", stringify!($name), ".dat")),
            }
            .obj
        };
    };
    ($name:ident: $type:ty = $epr:expr) => {
        const $name: $type = unsafe {
            union Transmute {
                bytes: [u8; { std::mem::size_of::<$type>() }],
                obj: $type,
            }
            Transmute {
                bytes: *include_bytes!(concat!("../static_data/", stringify!($name), ".dat")),
            }
            .obj
        };
    };
}
