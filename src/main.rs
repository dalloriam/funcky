use std::path::Path;

use funck::FunckLoader;

// TODO: More dynamic loading... :|
const PLUGIN_SO_FILE: &str = "./src/plugins/testfn/target/debug/libtestfn.so";

fn main() {
    assert!(Path::new(PLUGIN_SO_FILE).exists());

    let mut loader = FunckLoader::new();
    loader.load_funcktion(PLUGIN_SO_FILE).unwrap();
    loader.call("hello_fn").unwrap();
}
