/// export auto-implements the `Funcktion` trait and generates an FFI wrapper to construct the exported structure.
#[macro_export]
macro_rules! export {
    ($function_type:ty, $call_path:path, $str:tt) => {
        // Auto Funcktion implementation.
        // TODO: Support having a $call_path function returning CallResults
        impl $crate::Funcktion for $function_type {
            // Return the user-provided name as a `&'static str`.
            fn name(&self) -> &'static str {
                $str
            }
            fn _call_internal(&self) -> $crate::CallResult<()> {
                // Generate a panic handler for the exported function.
                // This is necessary because unwinding a panic across FFI boundaries is UB.
                match std::panic::catch_unwind(|| $call_path(self)) {
                    Ok(_) => Ok(()),
                    Err(e) => Err($crate::CallError::default()),
                }
            }
        }

        // Generated call for creating raw Funcktion instances.
        #[no_mangle]
        pub extern "C" fn _funck_create() -> *mut $crate::Funcktion {
            let constructor: fn() -> $function_type = <$function_type>::default;
            let obj = constructor();
            let boxed_obj: Box<$crate::Funcktion> = Box::new(obj);
            Box::into_raw(boxed_obj)
        }
    };
}
