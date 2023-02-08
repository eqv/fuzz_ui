use capstone::{arch,Capstone, Insn};

use capstone::arch::BuildsCapstone;
use capstone::arch::BuildsCapstoneSyntax;

use goblin::{error, Object};
use goblin::elf::Elf;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use crate::cofi::{Cofi,CofiType};

use snafu::{ensure, OptionExt};
use newtypes::error::*;

enum DisasmResult{
    InsCofi(Cofi),
    NextAddr(u64),
    Invalid(),
}


pub struct Mapping{
    base: u64,
    data: Vec<u8>,
}

pub struct TargetProg{
    pub wordsize: u8,
    mappings: Vec<Mapping>,
    disasm: Capstone,
}

impl TargetProg {
    pub fn new(path: &str, pie_base: Option<u64>) -> error::Result<Self> {
        let path = Path::new(path);
        let mut fd = File::open(path)?;
        let mut buffer = Vec::new();
        fd.read_to_end(&mut buffer)?;
        match Object::parse(&buffer)? {
            Object::Elf(elf) => {
                return Ok(Self::new_from_elf(&buffer, &elf, pie_base))
            },
            _ => { unimplemented!(); }
        }
    }


    fn get_cs_mode(wordwidth: u8) -> arch::x86::ArchMode{
        return match wordwidth {
            64 => arch::x86::ArchMode::Mode64,
            32 => arch::x86::ArchMode::Mode32,
            _ => unreachable!()
        }
    }

    fn new_capstone(wordwidth: u8) -> Capstone{
        return Capstone::new()
            .x86()
            .mode(Self::get_cs_mode(wordwidth))
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build().expect("unable to create capston instance");
    }

    fn new_from_elf(buffer: &[u8], elf: &Elf, pie_base: Option<u64>) -> Self {
        let wordsize = if elf.is_64 {64} else {32};
        let disasm = Self::new_capstone(wordsize);
        let mut res = Self{mappings: vec!(), wordsize, disasm};
        for proghdr in elf.program_headers.iter() {
            if proghdr.is_executable(){
                println!("{:?} {:?}",proghdr.p_vaddr, pie_base);
                assert!(proghdr.p_vaddr!=0 || pie_base.is_some(), "needs pie_base if that target is compiled PIE"); 
                assert!(proghdr.p_vaddr==0 || pie_base.is_none(), "you probably don't want to use pie_base if that target is not PIE"); 
                let base = proghdr.p_vaddr+pie_base.unwrap_or(0);
                let data = buffer[proghdr.file_range()].to_vec();
                res.mappings.push(Mapping{base, data});
            }
        }
        return res
    }

    pub fn get_mapping(&self, start_addr: u64) -> Option<&Mapping> {
        for m in self.mappings.iter() {
            if m.base <= start_addr && m.base + m.data.len() as u64 > start_addr {
                return Some(m)
            }
        }
        return None;
    }

    pub fn disassemble_range(&self, start_addr: u64, end: u64) -> Result<Vec<(u64, String)>,Error>{
        let mut res = vec!();
        if let Some(mapping) = self.get_mapping(start_addr){
            let mut next_addr = start_addr;
            while next_addr < end {
                let (size, string) = self.disassemble_instr(mapping, next_addr)?;
                res.push((next_addr, string));
                next_addr += size as u64;
            }
        }
        return Ok(res);
    }

    pub fn disassemble_instr(&self, mapping: &Mapping, start_addr: u64) -> Result<(usize, String),Error>{
        let offset = (start_addr-mapping.base) as usize;
        if let Ok(insns) = self.disasm.disasm_count(&mapping.data[offset..],start_addr,1){
            ensure!(insns.len() == 1, DisassemblyError{task:"get instructions"});
            for ins in insns.iter(){
                let mnem = ins.mnemonic().context(DisassemblyError{task:"get mnemonic"})?;
                let opstr = ins.op_str().context(DisassemblyError{task:"get opstr"})?;
                return Ok((ins.bytes().len(),format!("{} {}",mnem,opstr)));
            }
        }
        return Ok((1, format!("{:02x}", mapping.data[offset])));
    }

    pub fn disassemble_cofi(&self, start_addr: u64) -> Cofi{
        if let Some(mapping) = self.get_mapping(start_addr){
            let mut next_addr = start_addr;
            loop {
                //println!("disassemble at {:x}", next_addr);
                //validate!(mapping.base <= next_addr && next_addr < mapping.base+mapping.data.len() as u64,"start: {:x} leads to {:x}",start_addr, next_addr);
                match self.get_cofi_or_next_addr(mapping, next_addr) {
                    DisasmResult::InsCofi(cofi) => return cofi,
                    DisasmResult::Invalid() => return Cofi::invalid(next_addr),
                    DisasmResult::NextAddr(a) => next_addr = a,
                }
            }
        }
        return Cofi::invalid(start_addr);
    }

    fn cofi_get_direct_addr(&self, ins: &Insn) -> Option<u64>{
        if let Some(opstr) = ins.op_str(){
            if &opstr[0..2] == "0x"{
                return u64::from_str_radix(&opstr[2..], 16).ok();
            }
        }
        return None;
    }

    fn cofi_from_jmp(&self, ins: &Insn) -> Cofi{
        if let Some(addr) = self.cofi_get_direct_addr(ins){
            return Cofi::new(ins.address(), ins.bytes().len() as u64, CofiType::Jmp(addr));
        }
        return Cofi::new(ins.address(), ins.bytes().len() as u64, CofiType::JmpIndirect());

    }

    fn cofi_from_conditional_jmp(&self, ins: &Insn) -> Cofi{
        if let Some(addr) = self.cofi_get_direct_addr(ins){
            return Cofi::new(ins.address(), ins.bytes().len() as u64, CofiType::JmpCond(addr));
        }
        return Cofi::new(ins.address(), ins.bytes().len() as u64, CofiType::JmpCondIndirect());
    }

    fn cofi_from_call(&self, ins: &Insn) -> Cofi{
        if let Some(addr) = self.cofi_get_direct_addr(ins){
            return Cofi::new(ins.address(), ins.bytes().len() as u64, CofiType::Call(addr));
        }
        return Cofi::new(ins.address(), ins.bytes().len() as u64, CofiType::CallIndirect());
    }

    fn cofi_from_ret(&self, ins: &Insn) -> Cofi{
        return Cofi::new(ins.address(), ins.bytes().len() as u64, CofiType::Ret());
    }

    fn cofi_from_capston_instruction(&self, ins: &Insn) -> Option<Cofi>{
        if let Some(mnemonic) = ins.mnemonic() {
            //println!("disassemble {:x} {} {}",ins.address(), mnemonic, ins.op_str().unwrap_or(""));
            match mnemonic {
                "call" => return Some(self.cofi_from_call(ins)),
                "ret"|"retn" => return Some(self.cofi_from_ret(ins)),
                "jmp" => return Some(self.cofi_from_jmp(ins)),
                _ => {}
            }
            if mnemonic.chars().nth(0).unwrap() == 'j' {
                return Some(self.cofi_from_conditional_jmp(ins));
            }
        }
        return None
    }

    fn get_cofi_or_next_addr(&self, mapping: &Mapping, addr: u64) -> DisasmResult{
        let offset = (addr-mapping.base) as usize;
        //println!("disassemblying {:x} {:x} {:x}", self.data[offset], self.data[offset+1], self.data[offset+2]);
        if let Ok(insns) = self.disasm.disasm_count(&mapping.data[offset..],addr,1){
            assert!(insns.len() <= 1);
            for insn in insns.iter(){
                if let Some(cofi) = self.cofi_from_capston_instruction(&insn) {
                    return DisasmResult::InsCofi(cofi);
                }
                return DisasmResult::NextAddr(addr+insn.bytes().len() as u64);
            }
        } 
        return DisasmResult::Invalid();
    }
    //pub fn disassemble_bb_with_end(&self, start: u64, end: u64) -> Option<()> {
    //}
}



//use byteorder::{LittleEndian, ReadBytesExt};
//
//use std::fs::File;
//use std::io::prelude::*;
//
//use basicblock::BasicBlock;
//use cofi::{Cofi,CofiType};
//
//enum DisasmResult{
//    InsCofi(Cofi),
//    NextAddr(u64),
//    Invalid(),
//}
//
//pub struct CodeDump{
//    pub addr: u64,
//    pub data: Vec<u8>,
//    cs: Capstone
//}
//
//impl CodeDump{
//    pub fn new_from_path(path: &str) -> Self {
//        let mut file=File::open(path).unwrap();
//        let offset = file.read_u64::<LittleEndian>().expect("couldn't read target_code_dump header");
//        let mut data = vec!();
//        file.read_to_end(&mut data).expect("couldn't read target_code_dump content");
//        return CodeDump::new(offset, data, 64);
//    }
//
//    pub fn new(addr: u64, data: Vec<u8>, wordwidth: u8) -> Self {
//        let cs = CodeDump::new_capstone(wordwidth);
//        CodeDump{addr, data, cs}
//    }
//    
//
//}
//


#[cfg(test)]
mod tests {
    use crate::target::TargetProg;
    use crate::cofi::*;
    #[test]
    fn test_load() {
        let target = TargetProg::new("test_data/test").expect("couldn't load target");
        assert_eq!(target.disassemble_cofi(0x4006FC), Cofi{addr: 0x400709, size:0x5, ctype: CofiType::Call(0x400530)} );
    }
}



