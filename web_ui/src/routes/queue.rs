use std::io::prelude::*;
use std::fs::File;

use actix_web::{web, http, HttpResponse, Responder};
use crate::app_state::AppState;
use crate::newtypes::{InputID};

pub fn api_queue_view_input(info: web::Path<(usize,)>, data: web::Data<AppState> ) -> impl Responder {
    let fuzz = data.get();
    let input_id = InputID::new(info.0);
    if let Some(inp) = fuzz.inputs.get_input(&input_id) {
        if let Ok(mut f) = File::open(&inp.meta.path){
            let mut buffer = Vec::new();
            if let Ok(_) = f.read_to_end(&mut buffer){
                return HttpResponse::Ok().json(buffer);
            }
        }
    }
    return HttpResponse::new(http::StatusCode::NOT_FOUND);
}

