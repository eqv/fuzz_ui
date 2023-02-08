use crate::cfg::CFG;

use newtypes::Config;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};

pub struct Disassembler{
    ps: SyntaxSet,
    ts: ThemeSet,
}

impl Disassembler{
    pub fn new(config: &Config) -> Self{
        //let ps = SyntaxSet::load_defaults_newlines();
        let mut sb = SyntaxSet::load_defaults_newlines().into_builder();
        sb.add_from_folder(format!("{}/{}",config.dirs.ijon_base_dir,"syntax/"),true).expect("couldn't load additional syntax definitions");    
        let ps = sb.build();
        let ts = ThemeSet::load_defaults();
        let _syntax = ps.find_syntax_by_extension("nasm").unwrap();
        return Self{ps, ts};
    }

    pub fn disassemble_html(&self, start: u64, end: u64, cfg: &CFG) -> Vec<(String,String)>{
        let syntax = self.ps.find_syntax_by_extension("nasm").unwrap();
        let mut h = HighlightLines::new(syntax, &self.ts.themes["InspiredGitHub"]);
        let mut res = vec!();
        let mut last_addr = start;
        for addr in cfg.cofi_addrs_in_range(start, end){
            let bb = cfg.get_bb_from_any_addr(addr).expect("inconsistent cache state");
            if bb.start() > last_addr { res.push(("".to_string(),"--------------------------".to_string())); }
            last_addr = bb.last_addr()+1;
            for (addr, dis) in cfg.diassemble_bb(bb).unwrap().into_iter() {
                let ranges: Vec<(Style, &str)> = h.highlight(&dis, &self.ps);
                let html = styled_line_to_highlighted_html(&ranges[..], IncludeBackground::No);
                res.push((format!("{:08x}",addr),html))
            }
        }  
        return res;
    }
}