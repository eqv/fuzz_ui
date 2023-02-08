use std::collections::HashMap;
use std::cell::{Ref,RefCell};
use std::rc::Rc;

use crate::source_state::{SourceFileInfo,SourceFile};
use dump_srcmap::{MappedFilename,StartAddr, EndAddr};
use newtypes::{SrcID,LineID,LineNum};

//#[derive(Debug, Fail)]
//enum CovError {
//    #[fail(display = "could not parse elf using object crate {}", desc)]
//    FailedToParseObject {
//        desc: String,
//    },
//    #[fail(display = "No info found for: {}", addr)]
//    NoInfoFound {
//        addr: u64,
//    },
//    #[fail(display = "could not parse elf using elf crate {}", desc)]
//    FailedToParseElf {
//        desc: String,
//    },
//}

struct AddrToLineMapping {
    addr_to_page: HashMap<u64, Vec<Option<LineID>>>,
    //addr_to_lineid: HashMap<u64, LineID>,
    lineid_to_addrs: HashMap<LineID, Vec<u64>>,
}

impl AddrToLineMapping {
    pub fn new() -> Self {
        return Self{
            //addr_to_lineid: HashMap::new(),
            addr_to_page: HashMap::new(), 
            lineid_to_addrs: HashMap::new()};
    }

    pub fn add_addrs(&mut self, addrs: std::ops::Range<u64>, line: &LineID){
        let mut last_page_addr = addrs.start - (addrs.start%4096);
        let mut last_page = self.addr_to_page.entry(last_page_addr).or_insert_with(|| vec!(None; 4096));
        for i in addrs {
            let cur_page_addr = i - (i%4096);
            if cur_page_addr != last_page_addr {
                last_page_addr = cur_page_addr;
                last_page = self.addr_to_page.entry(last_page_addr).or_insert_with(|| vec!(None; 4096));
            }
            if last_page[(i%4096) as usize].is_none(){
                last_page[(i%4096) as usize] = Some(*line);
                self.lineid_to_addrs.entry(*line).or_insert_with(|| vec!()).push(i);
            }
        }
    }

    pub fn add_addr(&mut self, addr: u64, line: &LineID) {
        let page_addr = addr - (addr%4096);
        let page = self.addr_to_page.entry(page_addr).or_insert_with(|| vec!(None; 4096));
        let offset = (addr%4096) as usize;
        if page[offset].is_none(){
            page[offset] = Some(*line);
            self.lineid_to_addrs.entry(*line).or_insert_with(|| vec!()).push(addr);
        }
    }

    pub fn get(&self, addr: u64) -> Option<&LineID> {
        let offset = (addr%4096) as usize;
        let page_addr = addr - (addr%4096);
        return self.addr_to_page.get(&page_addr).and_then(|v| v[offset].as_ref());
    }

    pub fn get_addrs(&self, line: &LineID) -> Option<&Vec<u64>> {
        return self.lineid_to_addrs.get(&line);
    }
}

struct SrcIDToSourceMapping {
    source_id_to_source: HashMap<SrcID, SourceFile>,
}

impl SrcIDToSourceMapping {
    pub fn new() -> Self{
        return Self{
            source_id_to_source: HashMap::new(), 
        }
    }

    pub fn insert(&mut self, id: SrcID, path: MappedFilename) {
        if !self.source_id_to_source.contains_key(&id){
            self.source_id_to_source.insert(id, SourceFile::new(id, path));
        }
    }

    pub fn get(&self, id: SrcID) -> Option<&SourceFile> {
        return self.source_id_to_source.get(&id);
    }

    //pub fn get_src(&self, id: SrcID) -> &SourceFile {
    //    return self.source_id_to_source.get(&id).expect("invalid SrcId");
    //}

    pub fn get_src_mut(&mut self, id: SrcID) -> &mut SourceFile {
        return self.source_id_to_source.get_mut(&id).expect("invalid mut SrcId");
    }
    pub fn add_line(&mut self, line: &LineID) {
        self.get_src_mut(line.file()).add_reachable_line(line.num());
    }
}

pub struct SourceInfo {
    line_lookup: AddrToLineMapping,
    source_lookup: SrcIDToSourceMapping,
    bb_cache: HashMap<std::ops::Range<u64>,Vec<LineID>>,
}

impl SourceInfo{
    pub fn new() -> Self{
        let line_lookup = AddrToLineMapping::new();
        let source_lookup = SrcIDToSourceMapping::new();
        return Self{line_lookup, source_lookup, bb_cache: HashMap::new()};
    }

    pub fn load_from_data(&mut self, file_info: HashMap<SrcID, MappedFilename>, line_info: Vec<(StartAddr,EndAddr, SrcID, LineNum)>) {
        for (start,end,srcid,linenum) in line_info.iter(){
            let line =  LineID::new(*srcid, *linenum );
            self.source_lookup.insert(*srcid, file_info[srcid].clone());
            self.source_lookup.add_line(&line);
            self.line_lookup.add_addrs(*start..*end,&line)
        }
    }

    //pub fn new_from_pack(path: &str) -> Result<Self,Error>{
    //    let lines = File::open(format!("{}/lines.json",path))?;
    //    let filemap = File::open(format!("{}/filemap.json",path))?;
    //    let sources: Map<SrcId,string> = serde_json::from_reader(filemap).expect("error while reading filemap.json");
    //    let lines: Vec<(u64,u64,SrcID,LineID)> = serde_json::from_reader(filemap).expect("error while reading filemap.json");

    //    let mut line_lookup = AddrToLineMapping::new();
    //    for (start,end,srcid,lined) in sources.iter(){
    //        let line =  LineID::new(srcid, LineNum::new(linenum as usize) );
    //        for addr in start..end {
    //            line_lookup.add_addr(addr, &line);
    //        }
    //    }
    //    let mut source_lookup = SrcIDToSourceMapping::new();
    //    let mut res = Self{line_lookup, source_lookup};
    //}

    pub fn get_lines_and_set_covered<'a>(&'a mut self, range: std::ops::Range<u64>) -> &'a Vec<LineID>{
        if !self.bb_cache.contains_key(&range){
            let mut res = vec!();
            let mut last = None;
            for a in range.clone() {
                if let Some(l) = self.get_line(a).cloned(){
                    if last != Some(l){
                        res.push(l);
                        last = Some(l);
                        self.set_covered_line(l);
                    }
                }
            }
            self.bb_cache.insert(range.clone(), res);
        }
        return self.bb_cache.get(&range).unwrap();
    } 

    pub fn set_covered_line(&mut self, line: LineID){
        self.source_lookup.get_src_mut(line.file()).add_covered_line(line.num());
    }

    pub fn get_line(&self, addr: u64) -> Option<&LineID>{
        return self.line_lookup.get(addr);
    }
    pub fn get_addrs(&self, line: &LineID) -> Option<&Vec<u64>>{
        return self.line_lookup.get_addrs(line);
    }

    pub fn get_src(&self, src: SrcID) -> Option<&SourceFile> {
        return self.source_lookup.get(src);
    }

    pub fn is_reachable(&self, line: LineID) -> bool {
        if let Some(source_state) = self.source_lookup.get(line.file()) {
            if source_state.is_reachable(line.num()) {
                return true;
            }
        }
        return false;
    }

    pub fn num_reachable_lines(&self) -> usize{
        return self.source_lookup.source_id_to_source.values().map(|src| src.reachable_lines().len()).sum();
    }

    pub fn get_file_infos(&self) -> HashMap<SrcID, SourceFileInfo>{
        return self.source_lookup.source_id_to_source.iter().map(|(sid, file_state)| (*sid,SourceFileInfo::new(file_state))).filter(|(sid , file_state)| (file_state.covered_lines != 0)).collect::<HashMap<_,_>>();
    }
    pub fn files(&self) -> impl Iterator<Item = &SourceFile>{
        return self.source_lookup.source_id_to_source.values();
    }
}
