/// Lazily loads data from a file at runtime or generates the data if not found
///
/// Example:
/// const_data!(
///    pub TEST_DATA: type = complex_runtime_calculation()
/// );
macro_rules! runtime_data {
    ($filename:literal, pub static $name:ident: Box<$type:ty> = $expr:expr) => {
        pub static $name: once_cell::sync::Lazy<Box<$type>> = once_cell::sync::Lazy::new(|| {
            // size of the object
            const SIZE: usize = std::mem::size_of::<$type>();

            // get the filepath for the file
            let filepath = std::env::current_exe()
                .expect("executable filepath not found")
                .parent()
                .expect("executable not in a parent directory")
                .join($filename);

            // Try to load the file
            if let Ok(data_bytes) = std::fs::read(filepath.as_path()) {
                // ensure that the number of bytes in the file is correct
                if data_bytes.len() == SIZE {
                    // reinterpret the vector of bytes as a boxed type
                    return unsafe {
                        Box::from_raw(Box::into_raw(data_bytes.into_boxed_slice()) as *mut $type)
                    };
                }
            }

            use std::io::Write;
            // generate the object
            let data_obj: Box<$type> = $expr;
            // transmute the boxed object to boxed bytes
            let data_bytes = unsafe { Box::from_raw(Box::into_raw(data_obj) as *mut [u8; SIZE]) };
            // write the bytes
            let mut file =
                std::fs::File::create(filepath.as_path()).expect("unable to create data file");
            file.write_all(data_bytes.as_slice())
                .expect("unable to write data file");
            // convert the bytes back to the object
            let data_obj: Box<$type> =
                unsafe { Box::from_raw(Box::into_raw(data_bytes) as *mut $type) };

            // return the object
            data_obj
        });
    };
    ($filename:literal, static $name:ident: $type:ty = $expr:expr) => {
        static $name: once_cell::sync::Lazy<Box<$type>> = once_cell::sync::Lazy::new(|| {
            // size of the object
            const SIZE: usize = std::mem::size_of::<$type>();

            // get the filepath for the file
            let filepath = std::env::current_exe()
                .expect("executable filepath not found")
                .parent()
                .expect("executable not in a parent directory")
                .join($filename);

            // Try to load the file
            if let Ok(data_bytes) = std::fs::read(filepath.as_path()) {
                // ensure that the number of bytes in the file is correct
                if data_bytes.len() == SIZE {
                    // reinterpret the vector of bytes as a boxed type
                    return unsafe {
                        Box::from_raw(Box::into_raw(data_bytes.into_boxed_slice()) as *mut $type)
                    };
                }
            }

            use std::io::Write;
            // generate the object
            let data_obj: Box<$type> = $expr;
            // transmute the boxed object to boxed bytes
            let data_bytes = unsafe { Box::from_raw(Box::into_raw(data_obj) as *mut [u8; SIZE]) };
            // write the bytes
            let mut file =
                std::fs::File::create(filepath.as_path()).expect("unable to create data file");
            file.write_all(data_bytes.as_slice())
                .expect("unable to write data file");
            // convert the bytes back to the object
            let data_obj: Box<$type> =
                unsafe { Box::from_raw(Box::into_raw(data_bytes) as *mut $type) };

            // return the object
            data_obj
        });
    };
}

/// Generates lazy static data at runtime and saves to a file
///
/// Example:
/// const_data!(
///    pub TEST_DATA: type = complex_runtime_calculation()
/// );
#[cfg(feature = "gen-const-data")]
macro_rules! const_data {
    (pub $name:ident: $type:ty = $expr:expr) => {
        pub static $name: once_cell::sync::Lazy<Box<$type>> = once_cell::sync::Lazy::new(|| {
            union Transmute<'a> {
                bytes: &'a[u8; { std::mem::size_of::<$type>() }],
                obj: &'a$type,
            }

            use std::io::Write;

            let data = $expr;
            let mut file =
                std::fs::File::create(concat!("const_data/", concat!(stringify!($name), ".dat")))
                    .expect("unable to write static data");
            file.write_all(unsafe { Transmute { obj: data.as_ref() }.bytes })
                .expect("unable to write static data");
            data
        });
    };
    ($name:ident: $type:ty = $expr:expr) => {
        pub static $name: once_cell::sync::Lazy<Box<$type>> = once_cell::sync::Lazy::new(|| {
            union Transmute<'a> {
                bytes: &'a[u8; { std::mem::size_of::<$type>() }],
                obj: &'a$type,
            }

            use std::io::Write;

            let data = $expr;
            let mut file =
                std::fs::File::create(concat!("const_data/", concat!(stringify!($name), ".dat")))
                    .expect("unable to write static data");
            file.write_all(unsafe { Transmute { obj: data.as_ref() }.bytes })
                .expect("unable to write static data");
            data
        });
    };
}

/// Loads const data at compiletime from lazy static data generated at runtime
///
/// Example:
/// const_data!(
///    pub TEST_DATA: type = complex_runtime_calculation()
/// );
///
/// complex_runtime_calculation() must return Box<type>
#[cfg(not(feature = "gen-const-data"))]
macro_rules! const_data {
    (pub $name:ident: $type:ty = $epr:expr) => {
        pub const $name: $type = unsafe {
            union Transmute {
                bytes: [u8; { std::mem::size_of::<$type>() }],
                obj: $type,
            }
            Transmute {
                bytes: *include_bytes!(concat!("../../const_data/", stringify!($name), ".dat")),
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
                bytes: *include_bytes!(concat!("../../const_data/", stringify!($name), ".dat")),
            }
            .obj
        };
    };
}
