/// export defines the FFI wrapper around the `Box<dyn Funcktion>` structure defined manually in the plugin.
#[macro_export]
macro_rules! export {
    ($function_type:ty, $call_path:path, $str:tt) => {
        use $crate::error::ResultExt;

        impl $crate::Funcktion for $function_type {
            fn name(&self) -> &'static str {
                $str
            }
            fn _call_internal(&self) -> $crate::error::Result<()> {
                let v = std::panic::catch_unwind(|| $call_path(self));
                if v.is_err() {
                    Err(funck::error::Error::CallError)
                } else {
                    Ok(())
                }
            }
        }

        #[no_mangle]
        pub extern "C" fn _funck_create() -> *mut $crate::Funcktion {
            let constructor: fn() -> $function_type = <$function_type>::default;
            let obj = constructor();
            let boxed_obj: Box<$crate::Funcktion> = Box::new(obj);
            Box::into_raw(boxed_obj)
        }
    };
}
