use std::path::Path;

mod server;
use server::{FunckBuilder, FunckLoader};

const PLUGIN_PATH: &str = "../samples/testfn";

fn main() {
    assert!(Path::new(PLUGIN_PATH).exists());

    let mut builder = FunckBuilder::new();
    let shared_object = builder.build(PLUGIN_PATH).unwrap();

    let mut loader = FunckLoader::new();
    loader.load_funcktion(&shared_object).unwrap();
    match loader.call("hello_fn") {
        Ok(()) => println!("Call OK"),
        Err(e) => {
            println!("Error occured: {:?}", e);
        }
    }
    println!("DONE");
}
