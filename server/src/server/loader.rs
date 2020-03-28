use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use libloading::{Library, Symbol};

use funck::error::*;
use funck::Funcktion;

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

    pub fn load_funcktion<P: AsRef<OsStr>>(&mut self, dylib_file: P) -> Result<()> {
        let lib = Library::new(dylib_file.as_ref()).context(funck::error::LoadingError {
            path: PathBuf::from(dylib_file.as_ref()),
        })?;

        let funcktion: Box<dyn Funcktion> = unsafe {
            type FunckCreate = unsafe fn() -> *mut dyn Funcktion;
            let constructor: Symbol<FunckCreate> =
                lib.get(b"_funck_create")
                    .context(funck::error::LoadingError {
                        path: PathBuf::from(dylib_file.as_ref()),
                    })?;

            let boxed_raw = constructor();

            Box::from_raw(boxed_raw)
        };

        self.loaded_libraries
            .insert(String::from(funcktion.name()), lib);
        self.funcks
            .insert(String::from(funcktion.name()), funcktion);

        Ok(())
    }

    pub fn call(&mut self, function_name: &str) -> Result<()> {
        self.funcks
            .get(function_name)
            .ok_or_else(|| Error::NotFound {
                name: String::from(function_name),
            })?
            ._call_internal()?;
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
