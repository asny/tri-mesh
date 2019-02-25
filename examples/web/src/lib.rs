
use wasm_bindgen::prelude::*;

include!("../../morph.rs");

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue>
{
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    main();
    Ok(())
}
