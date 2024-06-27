/// Asserts to the compiler that a condition is always true, allowing it to skip checking it
#[macro_export]
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

// Slightly modified from const-array-init crate
#[macro_export]
macro_rules! const_arr {
    ([$TYPE:ty; $SIZE:expr], |$name:ident| $body:expr) => {{
        let mut arr = if !std::mem::needs_drop::<$TYPE>() {
            // NO DROP
            // This is safe because we overwrite every value of the array
            // And dropping the uninitialized values when new ones are assigned does nothing

            #[allow(unused_unsafe, invalid_value)]
            unsafe {
                std::mem::MaybeUninit::<[$TYPE; $SIZE]>::uninit().assume_init()
            }
        } else {
            // DROP
            let $name: usize = 0;
            let temp_item: $TYPE = $body;
            [temp_item; $SIZE]
        };

        // Initialize array with proper data from closure's body
        let mut $name = 0;
        while $name < $SIZE {
            arr[$name] = $body;
            $name += 1;
        }
        arr
    }};
    ([$TYPE:ty; $SIZE:expr], $body:expr) => {{
        static_assertions::const_assert!(!std::mem::needs_drop::<$TYPE>());

        let mut arr = {
            // NO DROP
            // This is safe because we overwrite every value of the array
            // And dropping the uninitialized values when new ones are assigned does nothing

            #[allow(unused_unsafe, invalid_value)]
            unsafe {
                std::mem::MaybeUninit::<[$TYPE; $SIZE]>::uninit().assume_init()
            }
        };

        // Initialize array with proper data from closure's body
        let mut i = 0;
        while i < $SIZE {
            arr[i] = $body;
            i += 1;
        }
        arr
    }};
}
