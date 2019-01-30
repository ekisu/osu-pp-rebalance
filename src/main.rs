#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

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
        let mut context : HashMap<String, String> = HashMap::new();
        println!("{}", results);
        context.insert(String::from("results"), results);
        Template::render("pp", &context)
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
