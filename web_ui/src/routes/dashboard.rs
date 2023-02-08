use actix_web::{web, http, HttpResponse, Responder};

use crate::app_state::AppState;
use fuzzing_state::{FuzzingState};

pub fn dashboard(_data: web::Data<AppState>) -> impl Responder {
    return HttpResponse::Found().header(http::header::LOCATION, "/static/dash.html").finish();
}

#[derive(Debug,Serialize, Deserialize)]
pub struct DashInfo{
    target_path: String,
    num_inputs: usize,
    num_lines: usize,
    num_covered: usize,
}

impl DashInfo{
    pub fn new(fuzz: &FuzzingState) -> Self{
        let target_path = fuzz.target_path.to_string();
        let num_inputs = fuzz.inputs.get_input_infos().len();
        let num_lines = fuzz.src_info.num_reachable_lines();
        let num_covered = fuzz.inputs.get_coverd_lines().values().map(|x| x.len()).sum();
        return Self{target_path, num_inputs, num_lines, num_covered}
    }
}

pub fn api_dash_info(data: web::Data<AppState>) -> impl Responder {
    let fuzz=data.get();
    return HttpResponse::Ok().json(DashInfo::new(&fuzz));
}

pub fn api_dash_set_info(item: web::Json<DashInfo> ,_data: web::Data<AppState>) -> impl Responder{
    println!("{:?}", item)
}