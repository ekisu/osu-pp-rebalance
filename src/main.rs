#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate handlebars;
extern crate ctrlc;
extern crate serde;

use rocket::Rocket;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

pub mod config_functions;
use config_functions::{
    api_key, load_save_results, minimal_force_interval, num_threads, results_file,
};
pub mod handlebars_helpers;
pub mod performance_calculator;
pub mod profile_cache;
pub mod profile_queue;

use performance_calculator::{simulate_play, SimulationParams};
use profile_cache::ProfileCache;
use profile_queue::{ProfileQueue, RequestStatus};
use rocket::response::Redirect;
use rocket::State;

#[get("/?<user>")]
fn index(user: Option<String>) -> Template {
    let mut _user = user.unwrap_or(String::new()).clone();
    _user.make_ascii_lowercase();

    let mut context: HashMap<String, String> = HashMap::new();
    context.insert("user".to_string(), _user.clone());

    Template::render("index", &context)
}

#[get("/pp?<user>")]
fn pp(cache: State<Arc<ProfileCache>>, mut user: String) -> Result<Template, Redirect> {
    user.make_ascii_lowercase();

    if let Some((results, _)) = cache.get(user.clone()) {
        Ok(Template::render("pp", &results))
    } else {
        Err(Redirect::to(uri!(index: user)))
    }
}

#[get("/pp_request?<user>&<force>")]
fn pp_request(
    cache: State<Arc<ProfileCache>>,
    queue: State<ProfileQueue>,
    mut user: String,
    force: Option<bool>,
) -> JsonValue {
    let _force = force.unwrap_or(false);
    user.make_ascii_lowercase();

    println!("PP-request for {}", user);
    // This logic is still a bit convoluted...
    match cache.get(user.clone()) {
        Some((_, time)) => {
            if !_force {
                return json!({ "status": "done" });
            }

            match time.elapsed() {
                Ok(elapsed) => {
                    if elapsed < minimal_force_interval() {
                        return json!({
                            "status": "cant_force",
                            "remaining": (minimal_force_interval() - elapsed).as_secs()
                        });
                    }
                }
                Err(_) => {}
            }
        }
        None => {}
    }

    queue.enqueue(user);
    json!({ "status": "accepted" })
}

#[get("/pp_check?<user>")]
fn pp_check(queue: State<ProfileQueue>, mut user: String) -> JsonValue {
    user.make_ascii_lowercase();

    if let Some(status) = queue.status(user) {
        match status {
            RequestStatus::Pending(pos) => json!( { "status": "pending", "pos": pos } ),
            RequestStatus::Calculating => json!( { "status": "calculating" } ),
            RequestStatus::Done => json!( { "status": "done" } ),
            RequestStatus::Error => json!( { "status": "error" } ),
        }
    } else {
        json!( { "status": "error" } )
    }
}

#[derive(Deserialize)]
struct SimulateData {
    beatmap_id: i64,
    params: SimulationParams,
}

#[post("/simulate", data = "<json_data>")]
fn simulate(json_data: Json<SimulateData>) -> JsonValue {
    let data = json_data.into_inner();
    println!("Simul request for {}", data.beatmap_id);
    match simulate_play(data.beatmap_id, data.params) {
        Ok(res) => json!( { "status": "ok", "results": res } ),
        Err(_) => json!( { "status": "error" } ),
    }
}

fn build_rocket(cache: Arc<ProfileCache>, queue: ProfileQueue) -> Rocket {
    rocket::ignite()
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                .register_helper("format_number", Box::new(handlebars_helpers::format_number));
            engines
                .handlebars
                .register_helper("has_mods", Box::new(handlebars_helpers::has_mods));
        }))
        .manage(cache)
        .manage(queue)
        .mount("/", routes![index])
        .mount("/", routes![pp])
        .mount("/", routes![pp_request])
        .mount("/", routes![pp_check])
        .mount("/", routes![simulate])
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
}

fn main() {
    if api_key() == "" {
        panic!("No api key was set! Exiting!")
    }

    let cache = Arc::new(ProfileCache::new(if load_save_results() {
        Some(results_file())
    } else {
        None
    }));

    if load_save_results() {
        cache.setup_save_results_handler(results_file());
    }

    let queue = ProfileQueue::new(cache.clone(), num_threads());

    build_rocket(cache, queue).launch();
}
