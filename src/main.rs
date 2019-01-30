#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

#[macro_use]
extern crate serde_derive;

extern crate serde;

use rocket_contrib::templates::Template;
use std::collections::HashMap;

pub mod config;
pub mod performance_calculator;

use performance_calculator::calculate_performance;

#[get("/")]
fn index() -> Template {
    let context : HashMap<&str, &str> = HashMap::new();
    Template::render("index", &context)
}

#[get("/pp?<user>")]
fn pp(user: String) -> Template {
    if let Ok(results) = calculate_performance(user) {
        Template::render("pp", &results)
    } else {
        let context : HashMap<String, String> = HashMap::new();
        Template::render("error", &context)
    }
}

fn main() {
    rocket::ignite().attach(Template::fairing())
                    .mount("/", routes![index])
                    .mount("/", routes![pp])
                    .launch();
}
