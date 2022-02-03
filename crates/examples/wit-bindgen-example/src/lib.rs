// interface imported from javascript
wit_bindgen_rust::import!("./crates/examples/wit-bindgen-example/interfaces/printer.wit");

// interface exported from rust compiled to Wasm
// wit_bindgen_rust::export!("./crates/examples/wit-bindgen-example/interfaces/stringfuncs.wit");
mod bindings;
use bindings::*;

/// In this case there is no state
struct State;

/// Implementation of the trait created by the `wit` file
impl stringfuncs::Stringfuncs for State {
    fn capitalize(s: String) -> String {
      printer::print(&format!("Capitalizing \"{}\"", s));
      s.to_uppercase()
    }
}