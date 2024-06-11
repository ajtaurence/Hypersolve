/// Asserts to the compiler that a condition is always true, allowing it to skip checking it
macro_rules! assert_unchecked {
    ($cond:expr) => {
        if !($cond) {
            if cfg!(debug_assertions) {
                panic!("unchecked assertion failed")
            }
            std::hint::unreachable_unchecked()
        }
    };
}

/// Loads the bytes from a file or generates the data file by serializing `value` using rkyv
pub(crate) fn load_or_generate_bytes<T, F: FnOnce() -> T>(f: F, filename: &str) -> Vec<u8>
where
    T: rkyv::Archive
        + rkyv::Serialize<
            rkyv::ser::serializers::CompositeSerializer<
                rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
                rkyv::ser::serializers::FallbackScratch<
                    rkyv::ser::serializers::HeapScratch<1024>,
                    rkyv::ser::serializers::AllocScratch,
                >,
                rkyv::ser::serializers::SharedSerializeMap,
            >,
        >,
{
    // get the filepath for the file
    let filepath = std::env::current_exe()
        .expect("executable filepath not found")
        .parent()
        .expect("executable not in a parent directory")
        .join(filename);

    // try to load the bytes from the file
    if let Ok(bytes) = std::fs::read(filepath.as_path()) {
        bytes
    } else {
        use std::io::Write;

        #[cfg(feature = "progress")]
        let pb = {
            // Create static multiprogress bar
            static PROGRESS_BAR: once_cell::sync::Lazy<indicatif::MultiProgress> =
                once_cell::sync::Lazy::new(indicatif::MultiProgress::new);

            // create progress bar instance
            let pb = PROGRESS_BAR.add(
                indicatif::ProgressBar::new_spinner()
                    .with_style(
                        indicatif::ProgressStyle::with_template("{msg}{spinner}")
                            .unwrap()
                            .tick_strings(&[".  ", ".. ", "...", "...", "..."]),
                    )
                    .with_message(format!("Generating {}", filename)),
            );

            pb.enable_steady_tick(std::time::Duration::from_millis(200));
            pb
        };

        let bytes = rkyv::to_bytes::<_, 1024>(&f()).expect("unable to serialize object to bytes");

        // write the bytes to the file
        let mut file =
            std::fs::File::create(filepath.as_path()).expect("unable to create data file");
        file.write_all(bytes.as_slice())
            .expect("unable to write data file");

        #[allow(clippy::let_and_return)]
        let bytes = bytes.to_vec();

        #[cfg(feature = "progress")]
        pb.finish_and_clear();

        bytes
    }
}

/// Loads data from a file at runtime and performs zero copy deserialization using rkyv.
/// If the file is not present then the data will be generated from the given function.
/// The actual type of the data will be the archived version of the type.
///
/// # Example
/// ```ignore
/// load_or_generate_data!(static DATA: type = { expr }, "filename.ext");
/// ```
macro_rules! load_or_generate_data {
    ($vis:vis static $name:ident: $type:ty = $expr:expr, $filename:literal) => {
        $vis static $name: once_cell::sync::Lazy<&<$type as rkyv::Archive>::Archived> =
            once_cell::sync::Lazy::new(|| {
                // generate the static bytes
                static BYTES: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
                    crate::common::macros::load_or_generate_bytes::<$type, _>(|| $expr, $filename)
                });

                // interpret an archived reference from the bytes
                unsafe { rkyv::archived_root::<$type>(&BYTES[..]) }
            });
    };
}

/// Loads const data at compiletime from lazy static data generated at runtime
///
/// # Example
/// ```ignore
/// const_data!(pub DATA: type = complex_runtime_calculation());
/// ```
/// where `complex_runtime_calculation()` returns `Box<type>`.
///
/// If `feature = "gen-const-data"` then `DATA` is a lazy static type initialized by `complex_runtime_calculation()`
/// which saves the data to disk inside `const_data` folder. If `feature = "gen-const-data"` is not active then `DATA`
/// will be a const, loaded from the data on the disk at compile-time.
#[cfg(feature = "gen-const-data")]
macro_rules! const_data {
    ($vis:vis $name:ident: $type:ty = $expr:expr) => {
        $vis static $name: once_cell::sync::Lazy<Box<$type>> = once_cell::sync::Lazy::new(|| {
            union Transmute<'a> {
                bytes: &'a[u8; { std::mem::size_of::<$type>() }],
                obj: &'a$type,
            }

            use std::io::Write;

            let data = $expr;
            std::fs::create_dir_all(concat!(env!("CARGO_MANIFEST_DIR"),
            "\\const_data")).expect("unable to create const_data directory");
            let mut file =
                std::fs::File::create(concat!(env!("CARGO_MANIFEST_DIR"),
                "\\const_data\\", stringify!($name), ".dat"))
                    .expect("unable to write const data");
            file.write_all(unsafe { Transmute { obj: data.as_ref() }.bytes })
                .expect("unable to write const data");
            data
        });
    };
}

/// Loads const data at compiletime from lazy static data generated at runtime
///
/// # Example
/// ```ignore
/// const_data!(pub DATA: type = complex_runtime_calculation());
/// ```
/// where `complex_runtime_calculation()` returns `Box<type>`.
///
/// If `feature = "gen-const-data"` then `DATA` is a lazy static type initialized by `complex_runtime_calculation()`
/// which saves the data to disk inside `const_data` folder. If `feature = "gen-const-data"` is not active then `DATA`
/// will be a const, loaded from the data on the disk at compile-time.
#[cfg(not(feature = "gen-const-data"))]
macro_rules! const_data {
    ($vis:vis $name:ident: $type:ty = $epr:expr) => {
        $vis const $name: $type = unsafe {
            union Transmute {
                bytes: [u8; { std::mem::size_of::<$type>() }],
                obj: $type,
            }
            Transmute {
                bytes: *include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "\\const_data\\",
                    stringify!($name),
                    ".dat"
                )),
            }
            .obj
        };
    };
}

/// Evaluates the macro on every comma separated value
macro_rules! for_each {
    ($macro:ident!($($x:ident),*)) => {
        $(
            $macro!($x);
        )*
    };
}
