#[macro_use] extern crate rocket;
use std::env::set_var;

use core::config;
use core::image;

mod core;
mod api;
mod common;


#[launch]
fn rocket_main() -> _ {

    set_var("RUST_LOG", "info");
    env_logger::init();

    let config = config::read_config().unwrap();
    
    let map = image::build_images(&config);
    
    info!("{:?}", map);
    
    rocket::build()
        .manage(config)
        .mount("/", routes![api::languages])
        .mount("/", routes![api::new_language])
        .mount("/", routes![api::remove_language])
        .mount("/", routes![api::run_code])
        .register("/", catchers![api::default])
        // .attach(api::response_fairing::ResponseFairing)
}
