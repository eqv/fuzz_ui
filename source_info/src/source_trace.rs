use std::collections::{HashMap, HashSet};

use crate::source_info::SourceInfo;
use crate::newtypes::{LineID, SrcID, LineNum, TransitionType, IsXref};
use crate::binary_cfg::{BinaryTrace, CFG};

#[derive(Debug)]
pub struct SourceTrace{
    pub coverage: HashMap<SrcID, HashSet<LineNum>>,
    pub transitions: HashMap<LineID, HashSet<(TransitionType,LineID,IsXref)> >,
}

impl SourceTrace{
    pub fn new() -> Self {
        return Self{coverage: HashMap::new(), transitions: HashMap::new()}
    }

    pub fn new_from_binary_trace(src_cfg: &mut SourceInfo, cfg: &CFG, bin: &BinaryTrace) -> Self{
        let mut trace = SourceTrace::new();
        for trans in bin.transitions.iter() {
            let bb = cfg.get_bb_from_any_addr(trans.dst).unwrap();
            for line in src_cfg.get_lines_and_set_covered(trans.dst..bb.exit()).iter(){
                trace.add_coverage(*line);
            }

            if trans.src != 0xffffffff_ffffffff && trans.dst != 0xffffffff_ffffffff{
                if let Some(line_src) = src_cfg.get_line(trans.src) {
                    if let Some(line_dst) = src_cfg.get_line(trans.dst) {
                        trace.add_transition(*line_src, *line_dst, trans.t_type);
                    }
                }
            }
        }
        return trace;
    }

    pub fn add_transition(&mut self, src: LineID, dst:LineID, ttype: TransitionType) {
        {
        let entry = self.transitions.entry(src).or_insert_with(|| HashSet::new());
        entry.insert((ttype, dst, IsXref::Direct));
        }
        {
        let entry = self.transitions.entry(dst).or_insert_with(|| HashSet::new());
        entry.insert((ttype, src, IsXref::Xref));
        }
    }

    pub fn add_coverage(&mut self, line: LineID) {
        let entry = self.coverage.entry(line.file()).or_insert_with(|| HashSet::new());
        entry.insert(line.num());
    }
    
    pub fn covers(&self, line: LineID) -> bool {
        return self.coverage.get(&line.file())
            .map(|lines| lines.contains(&line.num()))
            .unwrap_or(false);
    }

    pub fn transitions(&self, line: LineID) -> Vec<(TransitionType,LineID,IsXref)> {
        let mut res = vec!();
        if let Some(set) = self.transitions.get(&line) {
            for (t,l,x) in set.iter(){
                res.push((*t,*l,*x));
            }
        }
        return res;
    }

}

