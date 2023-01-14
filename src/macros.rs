#[cfg(feature = "gen-small-tables")]
macro_rules! const_lookup_table {
    ($name:ident: $type:ty = ($file:tt, $func:expr)) => {
        lazy_static::lazy_static! {
            static ref $name: $type = {
                use std::io::Write;

                let result = ($func)();
                let mut file = std::fs::File::create(concat!("luts/", $file)).expect("unable to write lookup table");
                file.write_all(bytemuck::cast_slice(result)).expect("unable to write lookup table");
                result
            };
        }
    };
}

#[cfg(not(feature = "gen-small-tables"))]
macro_rules! const_lookup_table {
    ($name:ident: $type:ty = ($file:tt, $func:expr)) => {
        const $name: $type = bytemuck::cast_slice(include_bytes!(concat!("../luts/", $file)));
    };
}
