use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::path::PathBuf;

use libloading::{Library, Symbol};

use funck::Funcktion;

use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("FFI Error calling function [{}]", name))]
    CallError {
        source: funck::CallError,
        name: String,
    },

    #[snafu(display("Failed to load funck shared object file [{}]: {}", path.display(), source))]
    FailedToLoadLibrary { source: io::Error, path: PathBuf },

    #[snafu(display(
        "{} symbol not found in shared object [{}]. Maybe you forgot an export!() macro invocation?",
        symbol, path.display()
    ))]
    MissingSymbol {
        source: io::Error,
        path: PathBuf,
        symbol: String,
    },

    #[snafu(display("Unknown function: {}", name))]
    UnknownFunction { name: String },
}

type Result<T> = std::result::Result<T, Error>;

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
        let lib = Library::new(dylib_file.as_ref()).context(FailedToLoadLibrary {
            path: PathBuf::from(dylib_file.as_ref()),
        })?;

        let funcktion: Box<dyn Funcktion> = unsafe {
            type FunckCreate = unsafe fn() -> *mut dyn Funcktion;
            const CTOR_SYMBOL: &[u8] = b"_funck_create";
            let constructor: Symbol<FunckCreate> = lib.get(CTOR_SYMBOL).context(MissingSymbol {
                path: PathBuf::from(dylib_file.as_ref()),
                symbol: String::from_utf8_lossy(CTOR_SYMBOL).to_string(),
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

    pub fn call(&self, function_name: &str) -> Result<()> {
        self.funcks
            .get(function_name)
            .ok_or(Error::UnknownFunction {
                name: String::from(function_name),
            })?
            ._call_internal()
            .context(CallError {
                name: String::from(function_name),
            })?;
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
