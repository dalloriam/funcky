use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};

use funck::{Funcktion, Request, Response};

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

struct LoadedFunck {
    pub funck: Box<dyn Funcktion>,
    pub lib: Library,
}

impl LoadedFunck {
    pub fn load<P: AsRef<Path>>(dylib_file: P) -> Result<LoadedFunck> {
        let lib = Library::new(dylib_file.as_ref()).context(FailedToLoadLibrary {
            path: PathBuf::from(dylib_file.as_ref()),
        })?;

        let funck: Box<dyn Funcktion> = unsafe {
            type FunckCreate = unsafe fn() -> *mut dyn Funcktion;
            const CTOR_SYMBOL: &[u8] = b"_funck_create";
            let constructor: Symbol<FunckCreate> = lib.get(CTOR_SYMBOL).context(MissingSymbol {
                path: PathBuf::from(dylib_file.as_ref()),
                symbol: String::from_utf8_lossy(CTOR_SYMBOL).to_string(),
            })?;

            let boxed_raw = constructor();

            Box::from_raw(boxed_raw)
        };

        log::debug!(
            "loaded funcktion <{}> from shared object [{}]",
            funck.name(),
            dylib_file.as_ref().display()
        );
        Ok(LoadedFunck { funck, lib })
    }
}

/// The FunckLoader manages all Funcks currently loaded, as well as their associated dylibs.
pub struct FunckLoader {
    funcks: HashMap<String, LoadedFunck>,
    lib_index: HashMap<String, String>,
}

impl FunckLoader {
    pub fn new() -> FunckLoader {
        FunckLoader {
            funcks: HashMap::new(),
            lib_index: HashMap::new(),
        }
    }

    pub fn load_funcktion<P: AsRef<Path>>(&mut self, dylib_file: P) -> Result<String> {
        log::debug!(
            "request load of shared object: {}",
            dylib_file.as_ref().to_string_lossy()
        );

        let library_name = dylib_file
            .as_ref()
            .file_stem()
            .unwrap_or_else(|| "libunknown".as_ref())
            .to_string_lossy()
            .to_string();

        self.unload_library(&library_name);

        let foreign_funck = LoadedFunck::load(dylib_file)?;

        let fn_name = String::from(foreign_funck.funck.name());

        self.lib_index.insert(library_name, fn_name.clone());
        self.funcks.insert(fn_name.clone(), foreign_funck);

        Ok(fn_name)
    }

    pub fn has(&self, function_name: &str) -> bool {
        self.funcks.contains_key(function_name)
    }

    pub fn call(&self, function_name: &str, request: Request) -> Result<Response> {
        self.funcks
            .get(function_name)
            .ok_or(Error::UnknownFunction {
                name: String::from(function_name),
            })?
            .funck
            ._call_internal(request)
            .context(CallError {
                name: String::from(function_name),
            })
    }

    fn unload_library(&mut self, library_name: &str) {
        if let Some(fnk_name) = self.lib_index.remove(library_name) {
            if let Some(fnk) = self.funcks.remove(&fnk_name) {
                // Force dropping of lib.
                drop(fnk);
                log::debug!("unloaded {}", fnk_name);
            }
        }
    }

    fn unload(&mut self) {
        self.lib_index.clear();
        for (fnk_n, fnk) in self.funcks.drain() {
            drop(fnk_n);
            drop(fnk);
        }
    }
}

impl Drop for FunckLoader {
    fn drop(&mut self) {
        if !self.funcks.is_empty() || !self.lib_index.is_empty() {
            self.unload();
        }
    }
}
