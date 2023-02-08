extern crate binary_cfg;
extern crate glob;
extern crate newtypes;
extern crate regex;
extern crate qemu_forkserver;
extern crate tempdir;
extern crate snafu;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::fs;
use std::collections::HashMap;

use tempdir::TempDir;
use regex::Regex;


use snafu::{ensure, ResultExt};
use newtypes::error::*;
use newtypes::{TransitionType, Config};

use binary_cfg::{BinaryTrace, CFG, BinaryTransition, RawTrace};

pub struct QemuTracer{
    frk: qemu_forkserver::ForkServer,
    tmp_dir: TempDir,
}

impl QemuTracer{
    pub fn new(config: &Config) -> Self{
        let tmp_dir = TempDir::new("example").expect("Couldn't create tempdir for forkserver");
        let outdir = tmp_dir.path().to_string_lossy().to_string();
        println!("outdir: {}", outdir);
        let hide_output = true;
        let path = format!("{}/afl_fast_bin_cov/afl-qemu-trace_{}", config.dirs.ijon_base_dir, config.target.machine.bits());
        let mut args = vec!("afl-qemu-trace".to_string(), config.target.path.to_string());
        args.append(&mut config.target.args.clone());
        let frk = qemu_forkserver::ForkServer::new(path, args, hide_output, outdir);
        return Self{frk, tmp_dir};
    }

    pub fn perform_raw_trace(&mut self, input_data: &[u8]) -> Result<RawTrace, Error> {
        let _status = self.frk.run(input_data)?;
        let path =  self.tmp_dir.path().to_string_lossy();
        let trace_paths = glob::glob(&format!("{}/trace_thread_*.qemu",path))
            .expect("couldn't parse glob pattern").filter_map(|r| r.ok());
        let trace_path_strings = trace_paths.map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>();
        ensure!(trace_path_strings.len() > 0, NoQemuOutput{});
        let trace = load_raw_thread_traces(&trace_path_strings)?;
        let trace_paths = glob::glob(&format!("{}/trace_thread_*.qemu",path))
            .expect("couldn't parse glob pattern").filter_map(|r| r.ok());
        for p in trace_paths {
            fs::remove_file(p.clone()).context(ReadQemuTrace{path: p})?;
        }
        return Ok(trace);
    }
}


pub fn get_trace_paths(trace_dir: &str) -> HashMap<String,Vec<String>>{
    let trace_paths = glob::glob(&format!("{}/input_*_thread_*.qemu", trace_dir)).expect("couldn't parse glob pattern").filter_map(|r| r.ok());
    let re = Regex::new(r"^input_(.*)_thread_\d+\.qemu$").unwrap();
    let mut res = HashMap::new();
    for p in trace_paths{
        let base = p.file_name();
        let mat = re.captures(base.unwrap().to_str().unwrap()).unwrap();
        let path = p.to_str().unwrap().to_string();
        let input_name = mat[1].to_string();
        (*res.entry(input_name).or_insert_with(|| vec!())).push(path); 
    }
    return res;
}


fn load_raw_trace(trace: &mut RawTrace, path: &String) -> Result<(), Error>{
    let f = File::open(path).context(ReadQemuTrace { path })?;
    let f = BufReader::new(f);
    for (i,line) in f.lines().enumerate() {
        let line = line.unwrap();
        let (last_bb, target_bb, _size) = parse_trace_line(&line)?;
        let cnt = 1;
        trace.add_transition( BinaryTransition::new(last_bb, target_bb, TransitionType::Unknown, cnt ) );
    }
    return Ok(());
}

pub fn load_raw_thread_traces(paths: &Vec<String>) -> Result<RawTrace, Error>{
    let mut trace = RawTrace::new();
    for path in paths.iter(){
        load_raw_trace(&mut trace, path)?;
    }
    return Ok(trace);
}

pub fn load_thread_traces(cfg: &mut CFG, paths: &Vec<String>) -> Result<BinaryTrace, Error>{
    let raw_trace = load_raw_thread_traces(paths)?;
    return Ok(cfg.diassemble_raw_trace(raw_trace));
}

fn parse_trace_line(line: &str) -> Result<(u64, u64, u16),Error> {
    let args = line.split(",").collect::<Vec<_>>();
    ensure!(args.len() == 3, ParseLineQemuTrace { line });
    let src = u64::from_str_radix(args[0], 16).context(ParseIntQemuTrace{line})?;
    let dst =  u64::from_str_radix(args[1], 16).context(ParseIntQemuTrace{line})?;
    let cnt = u16::from_str_radix(args[2], 10).context(ParseIntQemuTrace{line})?;
    return Ok((src,dst,cnt ));
}

#[cfg(test)]
mod tests {
    use binary_cfg::*;
    use newtypes::*;
    use crate::*;

    #[test]
    fn test_load_trace_file(){
        let target = TargetProg::new("test_data/test").expect("couldn't load target");
        assert_eq!(target.disassemble_cofi(0x4006FC), Cofi{addr: 0x400709, size:0x5, ctype: CofiType::Call(0x400530)} );

        let mut cfg = CFG::new(target);
        let trace = load_thread_traces(&mut cfg, &vec!("test_data/out/input_b_thread_31848.qemu".to_string())).expect("couldn't parse trace file");

        assert_eq!(trace.transitions[0], BinaryTransition::new(0xffffffff_ffffffff, 0x400550,  TransitionType::Entry, 1 ));
        assert_eq!(trace.transitions[1], BinaryTransition::new(0x400574, 0x400520, TransitionType::Call, 1));
    }

    #[test]
    fn test_tracer(){
        let path = "../afl_fast_bin_cov/workdir/test".to_string();
        let args = vec!();
        let target = TargetConfig::new_x86_64(path, args);
        let mut qemu = QemuTracer::new(&Config{target, dirs: Dirs{ijon_base_dir: "../".to_string(), source_dir: "".to_string() }});
        let raw_trace = qemu.perform_raw_trace(&"1\n2\n3".to_string().into_bytes()).expect("couldn't trace"); 
        println!("{:?}", raw_trace);
        assert!(raw_trace.transitions.len() > 0);
    }
}
