use rkyv::{ser::serializers::*, Archive, Serialize};

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
                    crate::data_loading::load_or_generate_bytes::<$type, _>(|| $expr, $filename)
                });

                // interpret an archived reference from the bytes
                unsafe { rkyv::archived_root::<$type>(&BYTES[..]) }
            });
    };
}

/// Loads the bytes from a file or generates the data file by serializing `value` using rkyv
pub(crate) fn load_or_generate_bytes<T, F: FnOnce() -> T>(f: F, filename: &str) -> Vec<u8>
where
    T: Archive
        + Serialize<
            CompositeSerializer<
                AlignedSerializer<rkyv::AlignedVec>,
                FallbackScratch<HeapScratch<1024>, AllocScratch>,
                SharedSerializeMap,
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

pub(crate) use load_or_generate_data;
