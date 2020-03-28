use std::collections::HashMap;
use std::ffi::OsStr;

use libloading::{Library, Symbol};

use rood::{Cause, CausedResult, Error};

use crate::Funcktion;

/// export defines the FFI wrapper around the `Box<dyn Funcktion>` structure defined manually in the plugin.
#[macro_export]
macro_rules! export {
    ($function_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _funck_create() -> *mut $crate::Funcktion {
            let constructor: fn() -> $function_type = $constructor;
            let obj = constructor();
            let boxed_obj: Box<$crate::Funcktion> = Box::new(obj);
            Box::into_raw(boxed_obj)
        }
    };
}

/// The FunckLoader manages all Funcks currently loaded, as well as their associated dylibs.
pub struct FunckLoader {
    funcks: HashMap<String, Box<dyn Funcktion>>,
    loaded_libraries: HashMap<String, Library>,
}

impl FunckLoader {
    pub fn new() -> FunckLoader {
        FunckLoader {
            funcks: HashMap::new(),
            loaded_libraries: HashMap::new(),
        }
    }

    pub fn load_funcktion<P: AsRef<OsStr>>(&mut self, dylib_file: P) -> CausedResult<()> {
        let lib = Library::new(dylib_file.as_ref())
            .map_err(|e| Error::new(Cause::IOError, &e.to_string()))?;

        let funcktion: Box<dyn Funcktion> = unsafe {
            type FunckCreate = unsafe fn() -> *mut dyn Funcktion;
            let constructor: Symbol<FunckCreate> = lib
                .get(b"_funck_create")
                .map_err(|e| Error::new(Cause::InvalidData, &e.to_string()))?;

            let boxed_raw = constructor();

            Box::from_raw(boxed_raw)
        };

        self.loaded_libraries
            .insert(String::from(funcktion.name()), lib);
        println!("Loaded funck: {}", funcktion.name());
        self.funcks
            .insert(String::from(funcktion.name()), funcktion);

        Ok(())
    }

    pub fn call(&mut self, function_name: &str) -> CausedResult<()> {
        if !self.funcks.contains_key(function_name) {
            return Err(Error::new(
                Cause::NotFound,
                &format!("Funck [{}] does not exist", function_name),
            ));
        }
        self.funcks
            .get(function_name)
            .ok_or_else(|| {
                Error::new(
                    Cause::NotFound,
                    &format!("Funck [{}] does not exist", function_name),
                )
            })?
            .call();
        Ok(())
    }

    fn unload(&mut self) {
        self.funcks.clear();

        for (lib_n, lib) in self.loaded_libraries.drain() {
            drop(lib_n);
            drop(lib);
        }
    }
}

impl Drop for FunckLoader {
    fn drop(&mut self) {
        if !self.funcks.is_empty() || !self.loaded_libraries.is_empty() {
            self.unload();
        }
    }
}
