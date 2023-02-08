use actix_web::{web, http, HttpResponse, Responder};

use std::collections::{HashMap, HashSet};

use crate::app_state::AppState;
use crate::newtypes::{LineID, SrcID};
use crate::highlight::get_hl_file_content;
use dump_srcmap::MappedFilename;
use binary_cfg::CofiType;
use regex::Regex;

#[derive(Serialize)]
struct CodeLineInfo {
    content: String,
    klass: &'static str,
    index: usize,
}


pub fn api_view_src(info: web::Path<(usize,)>, data: web::Data<AppState> ) -> impl Responder {
    let fuzz = data.get();
    let src_id = SrcID::new(info.0);
    if let Some(src) = fuzz.get_source(src_id) {
        if let MappedFilename::Known(path) = src.path(){
            let mut lines = vec![];
            for (index, content) in get_hl_file_content(path).iter().enumerate() {
                let klass = fuzz
                    .get_coverage_state(&LineID::new_i(src_id, index + 1))
                    .map(|cov| cov.css_class())
                    .unwrap_or("dead");
                lines.push(CodeLineInfo {
                    content: content.to_string(),
                    index: index + 1,
                    klass,
                });
            }
            return HttpResponse::Ok().json(lines);
        }
    }
    return HttpResponse::new(http::StatusCode::NOT_FOUND);
}

pub fn api_view_line_inputs(info: web::Path<(usize,usize)>, data: web::Data<AppState> ) -> impl Responder {
    let fuzz = data.get();
    let src_id = SrcID::new(info.0);
    if let Some(_src) = fuzz.get_source(src_id) {
        let data = fuzz.get_inputs(&LineID::new_i(src_id, info.1)).unwrap_or(vec!());
        return HttpResponse::Ok().json(data);
    }
    return HttpResponse::new(http::StatusCode::NOT_FOUND);
}

pub fn api_view_line_addrs(info: web::Path<(usize,usize)>, data: web::Data<AppState> ) -> impl Responder {
    let fuzz = data.get();
    let src_id = SrcID::new(info.0);
    println!("get addrs for {:?} {}",src_id, info.1);
    if let Some(_src) = fuzz.get_source(src_id) {
        println!("found source for id {:?} {}",src_id, info.1);
        let data = fuzz.src_info.get_addrs(&LineID::new_i(src_id, info.1))
        .map(|addrs| addrs.into_iter().map(|a| format!("{:08x}",a)).collect::<Vec<_>>() );
        return HttpResponse::Ok().json(data);
    }
    return HttpResponse::new(http::StatusCode::NOT_FOUND);
}

pub fn api_view_asm(info: web::Path<(String,)>, data: web::Data<AppState> ) -> impl Responder {
    let fuzz = data.get();
    if let Ok(min) = u64::from_str_radix(&info.0,16) {
        let base = min-(min%512);
        let start = base - 512;
        let end = base + 1024;
        return HttpResponse::Ok().json(fuzz.disassembler.disassemble_html(start, end, &fuzz.cfg));
     
    }
    return HttpResponse::new(http::StatusCode::BAD_REQUEST);
}


pub fn api_view_transitions(info: web::Path<(usize,)>, data: web::Data<AppState> ) -> impl Responder {
    let fuzz = data.get();
    let src_id = SrcID::new(info.0);
    if let Some(src) = fuzz.get_source(src_id) {
        if let MappedFilename::Known(path) = src.path(){
            let mut transitions = HashMap::new();
            for (index, _content) in get_hl_file_content(&path).iter().enumerate() {
                if let Some(t) = fuzz.get_transitions(&LineID::new_i(src_id, index + 1)) {
                    transitions.insert(index+1, t.to_json() );
                }
            }
            return HttpResponse::Ok().json(transitions);
        }
    }
    return HttpResponse::new(http::StatusCode::NOT_FOUND);
}

pub fn api_view_incomplete_branches(data: web::Data<AppState> ) -> impl Responder {
    let fuzz = data.get();
    let mut lines = fuzz.cfg.bbs()
    .filter(|bb| bb.cofi.ctype.is_conditional() && bb.successors.len() < 2)
    .filter_map(|bb| fuzz.src_info.get_line(bb.exit()))
    .collect::<HashSet<_>>()
    .into_iter().collect::<Vec<_>>();
    lines.sort();
    return HttpResponse::Ok().json(lines);
}

#[derive(Deserialize,Debug)]
pub struct SearchReqData{
    pattern: String,
    only_covered: bool,
}

pub fn api_search_pattern(info: web::Json<SearchReqData>, data: web::Data<AppState>) -> impl Responder{
    let fuzz = data.get();
    println!("got request {:?}",info);
    if let Ok(regex) = Regex::new(&info.pattern) {
        let mut res = vec!();
        for f in fuzz.src_info.files() {
            let mut matches = f.find_matches(&regex);
            if info.only_covered {
                matches = matches.into_iter().filter(|l| fuzz.is_line_covered(l)).collect::<Vec<_>>();
            } 
            res.append( &mut matches );
        }
        return HttpResponse::Ok().json(res);
    }
    return HttpResponse::new(http::StatusCode::BAD_REQUEST);
}

pub fn api_view_files(data: web::Data<AppState>) -> impl Responder {
    let fuzz = data.get();
    return HttpResponse::Ok().json(fuzz.get_file_infos());
}

pub fn api_view_inputs(data: web::Data<AppState>) -> impl Responder {
    let fuzz = data.get();
    return HttpResponse::Ok().json(fuzz.get_input_infos());
}
