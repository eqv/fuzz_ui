extern crate actix;
extern crate actix_web;
extern crate actix_web_actors;
extern crate actix_files;
extern crate env_logger;
extern crate snafu;

extern crate syntect;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rmpv;
extern crate regex;

extern crate glob;

extern crate clap;
extern crate base64;

extern crate newtypes;
extern crate qemu_trace_loader;
extern crate dump_srcmap;
extern crate binary_cfg;
extern crate source_info;

use actix::*;
use actix_files as fs;
use actix_web::{web, HttpServer, App};

mod websock;
mod app_state;
mod app_server;
mod highlight;
mod routes;
mod fuzzing_state;
mod trace_state;
mod queue_state;
mod qemu_runner;

use newtypes::{Config,TargetConfig};
extern crate rand;
use rand::Rng;
use rand::thread_rng;
//extern crate rmp_serde;
extern crate ron;

use std::fs::File;

pub fn main() {
    let addr ="127.0.0.1:8080";

    let sys = System::new("ijon webui");

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let config = Config::from_args();

    let mut state = fuzzing_state::FuzzingState::new(config);
    println!("bin loading done");
    state.load_src_mapping();
    println!("src mapping done");

    let state = app_state::AppState::new(state);
    let server = app_server::AppServer::new(state.clone()).start();
    //let qemu_worker = qemu_runner::QemuRunner::new(state.clone()).start();
    //let secret = (0..16).map(|_| std::char::from_digit(thread_rng().gen_range(0,16),16).unwrap() ).collect::<String>();
    println!("running on http://{}/", addr);

    HttpServer::new(move || {
        App::new()
            //.wrap(middleware::Logger::default())
            .data(state.clone())
            .data(server.clone())
            //.data(qemu_worker.clone())
            .data(web::PayloadConfig::new(1 << 25))
            .data(web::JsonConfig::default().limit(1<<25))
            .service(web::resource("/api/view_src/{file_id}").to(routes::api_view_src))
            .service(web::resource("/api/view_transitions/{file_id}").to(routes::api_view_transitions))
            .service(web::resource("/api/view_line_inputs/{file_id}/{line_num}").to(routes::api_view_line_inputs))
            .service(web::resource("/api/view_asm/{addr}").to(routes::api_view_asm))
            .service(web::resource("/api/view_line_addrs/{file_id}/{line_num}").to(routes::api_view_line_addrs))
            .service(web::resource("/api/view_files/").to(routes::api_view_files))
            .service(web::resource("/api/view_inputs/").to(routes::api_view_inputs))
            .service(web::resource("/api/search_branches/").to(routes::api_view_incomplete_branches))
            .service(web::resource("/api/search_pattern/").to(routes::api_search_pattern))
            .service(web::resource("/api/queue/{input_id}").to(routes::api_queue_view_input))
            .service(web::resource("/api/dash_info/").to(routes::api_dash_info))
            .service(web::resource("/api/dash_set_info/").to(routes::api_dash_set_info))
            .service(web::resource("/api/client_info/").to(routes::api_client_info))
            .service(web::resource("/api/client_set_info/").to(routes::api_client_set_info))
            .service(web::resource("/api/client_add_input/").to(routes::api_client_add_input))
            .service(web::resource("/api/client_add_coverage_and_input/").to(routes::api_client_add_coverage_and_input))
            .route("/", web::get().to(routes::dashboard))
            .service(web::resource("/ws/").route(web::get().to(websock::ws_create)))
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind(addr)
    .unwrap()
    .start();

    sys.run().expect("couldn't run Actor system");
}
