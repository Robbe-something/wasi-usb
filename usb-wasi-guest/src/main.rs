use wit_bindgen::generate;
use crate::component::usb::usb::get_context;

generate!({
    world: "guest",
    path: "../wit",
});

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match get_context() {
        Ok(ctx) => {
            println!("Context: {:?}", ctx);
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
    println!("Hello, world!");
}
