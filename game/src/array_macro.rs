macro_rules! array {
    ($init:expr; $cap:expr) => {
        {
            let mut array: [_; $cap] = unsafe { ::std::mem::uninitialized() };
            for elem in array.iter_mut() {
                ::std::mem::forget(::std::mem::replace(elem, $init));
            }
            array
        }
    }
}

