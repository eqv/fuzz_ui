use std::collections::{HashMap, HashSet};
use crate::newtypes::{InputID};
use source_info::{SourceTrace};
use binary_cfg::BinaryTrace;
use crate::newtypes::{SrcID, LineNum};

#[derive(Serialize)]
pub struct InputInfo{
    id: InputID,
    path: String,
}

impl InputInfo{
    pub fn new(state: &InputState) -> Self{
        return Self{id: state.meta.id, path: state.meta.path.clone()}
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct InputState {
    pub meta: InputMeta,
    pub bin_trace: BinaryTrace,
    pub src_trace: SourceTrace,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct InputMeta{
    pub id: InputID,
    pub path: String,
}

impl InputMeta{
    pub fn new(path: String) -> Self{
        return InputMeta{ id: InputID::new(0), path }
    }
}

//pub struct InputMetadata{
//    attention_execs: 9666,
//    attention_secs': 3.3201169967651367,
//    fav_bits': {10852: 0},
//    fav_factor': 0.06647109985351562,
//    id': 1,
//    level': 0,
//    new_bits': {},
//    new_bytes': {1299: 2,
//    payload_hash': -897482700,
//    payload_len': 53,
//    state': {'name': 'finished'}}
//}
//
// 'info': {'attention_execs': 1,
//          'attention_secs': 3.3855438232421875e-05,
//          'exit_reason': 'regular',
//          'method': 'import',
//          'parent': None,
//          'performance': 0.0008308887481689453,
//          'time': 1547657215.545589,
//          'trace': [[18446744073709551615L, 4272704, 2],

impl InputState {
    //pub fn new_from_msgpack(id: usize, path: &str, metadata_path: &str, src_cov: &SrcCoverage) -> Option<Self> {
    //    let f = File::open(metadata_path).unwrap();
    //    let mut reader = BufReader::new(f);
    //    let metadata = read_value(&mut reader).unwrap();
    //    let bin_trace = InputState::get_trace(metadata)?;
    //    let src_trace = bin_trace.calc_src_trace(src_cov);

    //    return Some(Self {
    //        id: InputID::new(id),
    //        path: path.to_string(),
    //        bin_trace,
    //        src_trace,
    //    });
    //}
    
    pub fn new(meta: InputMeta, bin_trace: BinaryTrace, src_trace: SourceTrace) -> Self {
        return Self {
            meta,
            bin_trace,
            src_trace,
        };
    }


    pub fn src_trace(&self) -> &SourceTrace{
        return &self.src_trace;
    }

    pub fn id(&self) -> InputID {
        return self.meta.id;
    }

    //fn get_value<'a>(key: &str, msgpack: &'a Value) -> Option<&'a Value> {
    //    if let Some(map) = msgpack.as_map() {
    //        for (k,v) in map.iter(){
    //            if k.as_str() == Some(key) {
    //                return Some(v);
    //            }
    //        }
    //    }
    //    return None;
    //}

    //fn get_trace(msgpack: Value) -> Option<BinaryTrace> {
    //    let mut bin_trace = BinaryTrace::new();
    //    let info = InputState::get_value("info", &msgpack)?;
    //    let trace = InputState::get_value("trace", &info)?.as_array()?;
    //    for val in trace.iter() {
    //        let tran = val.as_array()?;
    //        let src = tran.get(0)?.as_u64()?;
    //        let dst = tran.get(1)?.as_u64()?;
    //        let cnt = tran.get(2)?.as_u64()?;
    //        let ttype = tran.get(3)?.as_u64()?;
    //        bin_trace.add_transition(src,dst, TransitionType::from_u8(ttype as u8)?, cnt);
    //    }
    //    return Some(bin_trace);
    //}
}

pub struct InputStorage {  
    input_id_to_input: HashMap<InputID, InputState>,
}

impl InputStorage {
    pub fn new() -> Self {
        let input_id_to_input: HashMap<InputID, InputState> = HashMap::new();
        return Self{input_id_to_input}
    }

    pub fn insert_input(&mut self, mut input: InputState) -> InputID{
        let id = InputID::new(self.input_id_to_input.len()+1);
        input.meta.id = id;
        self.input_id_to_input.insert(id, input);
        return id;
    }

    pub fn inputs(&self) -> Vec<&InputState> {
        return self.input_id_to_input.values().collect::<Vec<_>>();
    }

    pub fn get_input_infos(&self) -> HashMap<InputID, InputInfo> {
        return self.input_id_to_input.iter().map(|(iid, input_state)| (*iid,InputInfo::new(input_state))).collect::<HashMap<_,_>>();
    }

    pub fn get_input(&self, inp: &InputID) -> Option<&InputState> {
        return self.input_id_to_input.get(inp);
    }

    pub fn get_coverd_lines(&self) -> HashMap<SrcID,HashSet<LineNum>>{
        let mut res = HashMap::new();
        for inp in self.input_id_to_input.values(){
            for (srcid,lines) in inp.src_trace.coverage.iter(){
                let dat = res.entry(*srcid).or_insert_with(|| HashSet::new());
                for l in lines.iter(){
                    (*dat).insert(*l);
                }
            }
        }
        return res;
    }
}
