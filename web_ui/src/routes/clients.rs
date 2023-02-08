use actix_web::{web, http, HttpResponse, Responder};
use actix::Addr;
use crate::app_state::AppState;
use crate::app_server::{AppServer,UpdateClients};
use crate::queue_state::{InputMeta};
use crate::qemu_runner::{TraceInput,QemuRunner};

#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct ClientInfo{
    pub id: Option<usize>,
    pub fuzz_type: String,
    pub ticks: usize,
    pub execs_per_second: f32,
    pub num_inputs: usize,
}

pub fn api_client_info(data: web::Data<AppState>) -> impl Responder {
    let fuzz=data.get();
    return HttpResponse::Ok().json(&fuzz.clients);
}

pub fn api_client_set_info(item: web::Json<ClientInfo> ,data: web::Data<AppState>, srv: web::Data<Addr<AppServer>>) -> impl Responder{
    if let Some(id) = data.get_mut().set_client(item.into_inner()){
        srv.do_send(UpdateClients("update_client_info".to_string()));
        return HttpResponse::Ok().json(id);
    }
    return HttpResponse::new(http::StatusCode::NOT_FOUND);
}

#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct AddInputInfo{
    pub path: String,
    pub raw_data: String,
}

pub fn api_client_add_input(item: web::Json<AddInputInfo> , runner: web::Data<Addr<QemuRunner>>) -> impl Responder{
    let info = item.into_inner();

    if let Ok(raw_data) = base64::decode(&info.raw_data){
        let meta = InputMeta::new(info.path);
        runner.do_send(TraceInput{meta, raw_data});
        return HttpResponse::new(http::StatusCode::OK);
    }
    return HttpResponse::new(http::StatusCode::BAD_REQUEST);
}


#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct AddInputCoverageInfo{
    pub path: String,
    pub raw_data: String,
    pub coverage: String,
}

fn parse_transitions(dat: &str) -> Vec<(u64,u64,u64)> {
    use std::u64;
    let mut res = vec!();
    for l in dat.lines() {
        let ls = l.split(",").collect::<Vec<_>>();
        if ls.len() == 3 {
            let src = u64::from_str_radix(&ls[0], 16).unwrap();
            let dst = u64::from_str_radix(&ls[1], 16).unwrap();
            let cnt = u64::from_str_radix(&ls[2], 16).unwrap();
            res.push( (src,dst,cnt) )
        }
    }
    return res;
}

pub fn api_client_add_coverage_and_input(item: web::Json<AddInputCoverageInfo>, fuzz: web::Data<AppState>) -> impl Responder{
    use crate::binary_cfg::{BinaryTransition, RawTrace};
    use crate::newtypes::TransitionType;

    let info = item.into_inner();
    if let Ok(raw_data) = base64::decode(&info.raw_data){
        let meta = InputMeta::new(info.path);
        let mut raw_trace = RawTrace::new();
        for (src,dst,cnt) in parse_transitions(&info.coverage){
            raw_trace.add_transition(BinaryTransition::new(src,dst,TransitionType::Unknown,cnt));
        }
        fuzz.get_mut().insert_input_from_raw(meta, raw_trace);
        fuzz.get_mut().cfg.perform_recursive_disassembly();
        return HttpResponse::new(http::StatusCode::OK);
    }
    return HttpResponse::new(http::StatusCode::BAD_REQUEST);
}
