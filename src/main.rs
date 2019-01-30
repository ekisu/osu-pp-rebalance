#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

extern crate serde;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use rocket_contrib::json::JsonValue;
use std::collections::HashMap;

pub mod config;
use config::NUM_THREADS;
pub mod performance_calculator;
pub mod player_cache;

use performance_calculator::calculate_performance;
use player_cache::{PlayerCache, CalcStatus};
use rocket::State;

#[get("/")]
fn index() -> Template {
    let context : HashMap<&str, &str> = HashMap::new();
    Template::render("index", &context)
}

#[get("/pp?<user>")]
fn pp(cache: State<PlayerCache>, user: String) -> Template {
    if let Some(results) = cache.get_performance(user) {
        Template::render("pp", &results)
    } else {
        let context : HashMap<String, String> = HashMap::new();
        Template::render("error", &context)
    }
}

#[get("/pp_request?<user>")]
fn pp_request(cache: State<PlayerCache>, user: String) -> JsonValue {
    println!("PP-request for {}", user);
    if let Some(_) = cache.calculate_request(user) {
        json!({ "status": "done" })
    } else {
        json!({ "status": "accepted" })
    }
}

#[get("/pp_check?<user>")]
fn pp_check(cache: State<PlayerCache>, user: String) -> JsonValue {
    if let Some(status) = cache.check_status(user.clone()) {
        if let CalcStatus::Pending(pos, _) = status {
            let cur = cache.get_current_in_queue();
            cache.ping(user.clone());
            json!( { "status": "pending", "pos": pos - cur } )
        } else if status == CalcStatus::Calculating {
            json!( { "status": "calculating" } )
        } else if status == CalcStatus::Done {
            json!( { "status": "done" } )
        } else {
            json!( { "status": "error" } )
        }
    } else {
        json!( { "status": "error" } )
    }
}

fn main() {
    rocket::ignite().attach(Template::fairing())
                    .manage(PlayerCache::new(NUM_THREADS))
                    .mount("/", routes![index])
                    .mount("/", routes![pp])
                    .mount("/", routes![pp_request])
                    .mount("/", routes![pp_check])
                    .mount("/static", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
                    .launch();
}
