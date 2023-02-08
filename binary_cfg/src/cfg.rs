use std::collections::{HashMap, HashSet};
use memrange::Range;
use theban_interval_tree::IntervalTree;

use newtypes::error::*;
use newtypes::TransitionType;

use crate::cofi::{Cofi,CofiType};
use crate::target::TargetProg;
use crate::binary_trace::{BinaryTrace, RawTrace};

#[derive(Debug)]
pub struct BB {
    pub entries: HashSet<u64>,
    pub cofi: Cofi,
    pub successors: HashSet<u64>,
    pub predecessors: HashSet<u64>,
}

impl BB{
    pub fn new(addr: u64, cofi: Cofi) -> Self{
        let mut entries = HashSet::new();
        entries.insert(addr);
        let successors = HashSet::new();
        let predecessors = HashSet::new();
        return Self{entries, cofi, successors, predecessors};
    }

    pub fn add_entry(&mut self, addr: u64){
        self.entries.insert(addr);
    }

    pub fn start(&self) -> u64{
        return *self.entries.iter().min().expect("found bb with no entry");
    }

    pub fn exit(&self) -> u64 { 
        return self.cofi.addr;
    }

    pub fn last_addr(&self) -> u64 {
        return self.cofi.addr+self.cofi.size;
    }

    pub fn transition_type(&self, target: u64) -> TransitionType{
        if target == 0xffffffff_ffffffff {
            return TransitionType::OutOfTrace
        }
        if self.cofi.addr == 0xffffffff_ffffffff {
            return TransitionType::Entry
        }
        return match self.cofi.ctype{
            CofiType::JmpCond(_) => if self.cofi.addr + self.cofi.size == target  {TransitionType::NotTaken} else {TransitionType::Taken},
            CofiType::JmpCondIndirect() => if self.cofi.addr + self.cofi.size == target  {TransitionType::NotTaken} else {TransitionType::Taken},
            CofiType::JmpIndirect() => TransitionType::Indirect,
            CofiType::Jmp(_) => TransitionType::Direct,
            CofiType::Ret() => TransitionType::Ret,
            CofiType::Call(_) => TransitionType::Call,
            CofiType::CallIndirect() => TransitionType::Call,
            CofiType::Invalid() => TransitionType::Unknown,
        }
    }
}


pub struct CFG{
    target: TargetProg,
    addr_to_exit_addr: HashMap<u64, u64>,
    exit_addr_to_bb: HashMap<u64, BB>,
    bb_range_lookup: IntervalTree<u64>,
    known_disasembled_bbs: HashSet<u64>,
}

impl CFG{
    pub fn new(target:  TargetProg) -> Self {
        let mut addr_to_exit_addr = HashMap::new();
        let mut exit_addr_to_bb = HashMap::new();
        let oot = 0xffffffff_ffffffff;
        let size = 0;
        let ctype = CofiType::Invalid();
        let bb = BB::new(oot, Cofi::new(oot, size, ctype));
        let bb_range_lookup = IntervalTree::new();
        addr_to_exit_addr.insert(oot, oot);
        exit_addr_to_bb.insert(oot, bb);
        return Self{target, addr_to_exit_addr, exit_addr_to_bb, bb_range_lookup, known_disasembled_bbs: HashSet::new()};
    }

    pub fn perform_recursive_disassembly(&mut self) {
        self.disassemble_recursive( self.addr_to_exit_addr.keys().map(|x| *x).collect::<Vec<_>>() );
    }

    pub fn disassemble_recursive(&mut self, mut entries: Vec<u64>) {
        while let Some(addr) = entries.pop() {
            if self.known_disasembled_bbs.contains( &addr) {continue; }
            self.add_bb(addr);
            for dst in self.get_bb_from_any_addr(addr).unwrap().cofi.get_successors() {
                if !self.knows_bb(dst) {
                    entries.push(dst);
                }
                self.add_raw_transition(addr, dst);
            }
            self.known_disasembled_bbs.insert(addr);
        }
    }

    pub fn knows_bb(&self, addr: u64) -> bool{ 
        return self.addr_to_exit_addr.contains_key(&addr);
    }

    pub fn diassemble_raw_trace(&mut self, raw: RawTrace) -> BinaryTrace {
        let mut transitions = raw.transitions;
        for tr in transitions.iter_mut() {
            self.add_raw_transition(tr.src, tr.dst);
            let bb = self.get_bb_from_any_addr(tr.src).unwrap();
            let src_cofi_addr = bb.exit();
            tr.src = src_cofi_addr;
            tr.t_type = bb.transition_type(tr.dst);
        }
        return BinaryTrace{transitions};
    }

    pub fn get_cofi_addr(&self, entry: u64) -> Option<u64> {
        return self.addr_to_exit_addr.get(&entry).copied();
    }

    pub fn add_bb(&mut self, addr: u64){
        if !self.knows_bb(addr){
            let cofi = self.target.disassemble_cofi(addr);
            let bb = self.exit_addr_to_bb.entry(cofi.addr).or_insert_with(|| BB::new(addr, cofi) );
            self.bb_range_lookup.delete(Range::new(bb.start(),bb.last_addr()));
            bb.add_entry(addr);
            self.addr_to_exit_addr.insert(addr, bb.exit());
            self.addr_to_exit_addr.insert(bb.exit(), bb.exit());
            self.bb_range_lookup.insert(Range::new(bb.start(),bb.last_addr()), bb.exit());
        }
    }

    //src_bb can be either the address of the cofi or an entry point of the bb
    pub fn add_raw_transition(&mut self, src_bb_addr: u64, dst_bb_addr: u64){
        println!("TRANSITION {:x}->{:x}",src_bb_addr, dst_bb_addr);
        self.add_bb(dst_bb_addr);
        let src_cofi_addr = self.get_bb_from_any_addr_mut(src_bb_addr).expect("couldn't add transition to previously unseen BB").exit();
        self.get_bb_from_any_addr_mut(src_bb_addr).expect("couldn't add transition to previously unseen BB").successors.insert(dst_bb_addr);
        self.get_bb_from_any_addr_mut(dst_bb_addr).expect("couldn't add transition to previously unseen BB").predecessors.insert(src_cofi_addr);
    }
    
    //addr can be either the address of the cofi or an entry point of the bb
    pub fn get_bb_from_any_addr(&self, addr: u64) -> Option<&BB> {
        return self.addr_to_exit_addr.get(&addr).map(|a| *a).and_then(move |addr| self.exit_addr_to_bb.get(&addr));
    }

    //addr can be either the address of the cofi or an entry point of the bb
    pub fn get_bb_from_any_addr_mut(&mut self, addr: u64) -> Option<&mut BB> {
        return self.addr_to_exit_addr.get(&addr).map(|a| *a).and_then(move |addr| self.exit_addr_to_bb.get_mut(&addr));
    }

    pub fn bbs(&self) -> impl Iterator<Item = &BB>{
        return self.exit_addr_to_bb.values();
    }

    pub fn cofi_addrs_in_range<'a>(&'a self, from: u64, to:u64) -> impl Iterator<Item = u64> +'a{
        for (r,exit) in self.bb_range_lookup.iter(){
        }
        return self.bb_range_lookup.range(from, to).map(|(_,exit)| *exit);
    }

    pub fn diassemble_bb(&self, bb: &BB) -> Result<Vec<(u64, String)>, Error>{
        return self.target.disassemble_range(bb.start(), bb.last_addr());
    }
}
