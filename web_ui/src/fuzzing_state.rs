use std::collections::HashMap;

use binary_cfg::{TargetProg,CFG, RawTrace, Disassembler};
use source_info::{SourceFile, SourceFileInfo, SourceTrace,SourceInfo};
use crate::newtypes::{LineID, SrcID, InputID, CoverageCounts};
use crate::queue_state::{InputStorage, InputState, InputInfo, InputMeta};
use crate::trace_state::LineTransitions;
use routes::clients::ClientInfo;
use newtypes::Config;
use dump_srcmap;

pub struct FuzzingState {
    pub target_path: String,
    pub inputs: InputStorage,
    pub cfg: CFG,
    pub src_info: SourceInfo,
    pub disassembler: Disassembler,
    pub clients: Vec<ClientInfo>,
    pub config: Config,
}

impl FuzzingState {
    pub fn new(config: Config) -> Self {
        let inputs = InputStorage::new();
        let src_info = SourceInfo::new();
        let target = TargetProg::new(&config.target.path, config.target.pie_base).expect("couldn't load target file");
        let cfg = CFG::new(target);
        let target_path = config.target.path.to_string();
        let clients = vec!();
        let disassembler = Disassembler::new(&config);

        return Self {
            target_path,
            src_info,
            inputs,
            cfg,
            clients,
            config,
            disassembler,
        };
    }

    pub fn load_src_mapping(&mut self){
        let (file_info, line_info) = dump_srcmap::get_line_info(&self.target_path, self.config.target.pie_base);
        if file_info.len() == 0 {
            println!("WARNING: no source maps found in binary");
        }
        let file_info = dump_srcmap::find_best_src_files(file_info, &self.config.dirs.source_dir);
        self.src_info.load_from_data(file_info, line_info);
    }

    // pub fn load_inputs_with_qemu_traces(&mut self, in_dir: &str, trace_dir: &str){
    //     let input_to_trace = qemu_trace_loader::get_trace_paths(trace_dir);
    //     for input in glob::glob(&format!("{}/**/*",in_dir)).expect("could not parse glob pattern"){
    //         if let Ok(input) = input{
    //             self.load_input_with_qemu_trace(&input, &input_to_trace);
    //         }
    //     }
    // }

    // pub fn load_input_with_qemu_trace(&mut self, input: &std::path::Path, input_to_traces: &HashMap<String,Vec<String>>){
    //     let file_name = input.file_name().unwrap().to_str().unwrap();
    //     let path = input.to_str().unwrap();
    //     let meta = InputMeta::new(path.to_string());
    //     if let Some(traces) = input_to_traces.get(file_name){
    //         let raw_trace = qemu_trace_loader::load_raw_thread_traces(traces).unwrap();
    //         self.insert_input_from_raw(meta, raw_trace);
    //     } else {
    //         //println!("WARNING: no traces for {}", path);
    //     }
    // }

    pub fn insert_input_from_raw(&mut self, meta: InputMeta, raw_trace: RawTrace) {
        let bin_trace = self.cfg.diassemble_raw_trace(raw_trace);
        let src_trace = SourceTrace::new_from_binary_trace(&mut self.src_info, &self.cfg, &bin_trace);
        let inp = InputState::new(meta, bin_trace, src_trace);
        println!("got input {}", inp.meta.path);
        self.inputs.insert_input(inp);
    } 

    pub fn get_source(&self, src: SrcID ) -> Option<&SourceFile>{
        return self.src_info.get_src(src);
    }

    pub fn get_file_infos(&self) -> HashMap<SrcID, SourceFileInfo>{
        return self.src_info.get_file_infos();
    }

    pub fn get_input_infos(&self) -> HashMap<InputID, InputInfo>{
        return self.inputs.get_input_infos();
    }

    //THIS IS STUPID AS FUCK, REPLACE WITH CACHED O(1) SOLUTION!!!!!
    pub fn get_coverage_state(&self, line: &LineID) -> Option<CoverageCounts> {
        if !self.src_info.is_reachable(*line) {
            return None;
        }
        let mut res = CoverageCounts::new();
        for input in self.inputs.inputs(){
            if input.src_trace().covers(*line) {
                res.num_a += 1;
            }
        }
        return Some(res);
    }

    pub fn is_line_covered(&self, line: &LineID) -> bool {
        if !self.src_info.is_reachable(*line) {
            return false;
        }
        for input in self.inputs.inputs(){
            if input.src_trace().covers(*line) {
                return true;
            }
        }
        return false;
    }

    pub fn get_inputs(&self, line: &LineID) -> Option<Vec<InputID>>{
        if !self.src_info.is_reachable(*line) {
            return None;
        }
        let mut res = vec!();
        for input in self.inputs.inputs() {
            if input.src_trace().covers(*line){
                res.push(input.id());
            }
        }
        return Some(res)
    }

    //THIS IS STUPID AS FUCK, REPLACE WITH CACHED O(1) SOLUTION!!!!!
    pub fn get_transitions(&self, line: &LineID) -> Option<LineTransitions> {
        if !self.src_info.is_reachable(*line) {
            return None;
        }
        let mut res = LineTransitions::new();

        for input in self.inputs.inputs() {
            for (t,l,x) in input.src_trace().transitions(*line){
                res.add_transition(t,l,x);
            }
        }
        return Some(res);
    }

    pub fn set_client(&mut self, mut data: ClientInfo) -> Option<usize>{
        if let Some(i) = data.id {
            if i < self.clients.len(){
                self.clients[i]=data;
                return Some(i);
            }
            return None;
        }
        data.id = Some(self.clients.len());
        self.clients.push(data);
        return Some(self.clients.len()-1);
    }

}
