#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate handlebars;
extern crate serde;
extern crate ctrlc;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use rocket_contrib::json::{JsonValue, Json};
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

pub mod config;
use config::{NUM_THREADS, RESULTS_FILE_STORAGE, LOAD_SAVE_RESULTS};
pub mod performance_calculator;
pub mod player_cache;
pub mod handlebars_helpers;

use performance_calculator::{calculate_performance, simulate_play, SimulationParams};
use player_cache::{PlayerCache, CalcStatus, EnqueueResult};
use rocket::State;
use rocket::response::{Responder, Redirect};

#[get("/?<user>")]
fn index(user: Option<String>) -> Template {
    let mut _user = user.unwrap_or(String::new()).clone();
    _user.make_ascii_lowercase();

    let mut context : HashMap<String, String> = HashMap::new();
    context.insert("user".to_string(), _user.clone());

    Template::render("index", &context)
}

#[get("/pp?<user>")]
fn pp(cache: State<PlayerCache>, mut user: String) -> Result<Template, Redirect> {
    user.make_ascii_lowercase();

    if let Some(results) = cache.get_performance(user.clone()) {
        Ok(Template::render("pp", &results))
    } else {
        Err(Redirect::to(uri!(index: user)))
    }
}

#[get("/pp_request?<user>&<force>")]
fn pp_request(cache: State<PlayerCache>, mut user: String, force: Option<bool>) -> JsonValue {
    let _force = force.unwrap_or(false);
    user.make_ascii_lowercase();

    println!("PP-request for {}", user);
    match cache.calculate_request(user, _force) {
        EnqueueResult::AlreadyDone => json!({ "status": "done" }),
        EnqueueResult::Enqueued => json!({ "status": "accepted" }),
        EnqueueResult::CantForce(remaining_cooldown) => json!({ "status": "cant_force", "remaining": remaining_cooldown.as_secs() })
    }
}

#[get("/pp_check?<user>")]
fn pp_check(cache: State<PlayerCache>, mut user: String) -> JsonValue {
    user.make_ascii_lowercase();

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

#[derive(Deserialize)]
struct SimulateData {
    beatmap_id: i64,
    params: SimulationParams
}

#[post("/simulate", data = "<json_data>")]
fn simulate(json_data: Json<SimulateData>) -> JsonValue {
    let data = json_data.into_inner();
    println!("Simul request for {}", data.beatmap_id);
    match simulate_play(data.beatmap_id, data.params) {
        Ok(res) => json!( { "status": "ok", "results": res } ),
        Err(_) => json!( { "status": "error" } )
    }
}

fn main() {
    let cache = PlayerCache::new(NUM_THREADS,
        if LOAD_SAVE_RESULTS {
            Some(RESULTS_FILE_STORAGE)
        } else { 
            None
        });

    if LOAD_SAVE_RESULTS {
        cache.setup_save_results_handler(RESULTS_FILE_STORAGE);
    }

    rocket::ignite()
    .attach(Template::custom(|engines| {
        engines.handlebars.register_helper("format_number", Box::new(handlebars_helpers::format_number));
    }))
    .manage(cache)
    .mount("/", routes![index])
    .mount("/", routes![pp])
    .mount("/", routes![pp_request])
    .mount("/", routes![pp_check])
    .mount("/", routes![simulate])
    .mount("/static", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
    .launch();
}
