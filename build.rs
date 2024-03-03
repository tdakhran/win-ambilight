use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("arduino/led_layout.hpp")
        .translate_enum_integer_types(true)
        .enable_cxx_namespaces()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("led_layout.rs"))
        .expect("Couldn't write bindings!");
}



