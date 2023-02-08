use std::collections::{HashSet};
use std::path::Path;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

use regex::Regex;
use dump_srcmap::MappedFilename;
use crate::newtypes::{SrcID, LineID, LineNum};


#[derive(Serialize)]
pub struct SourceFileInfo{
    pub base_name: String,
    pub path: String,
    pub lines: usize,
    pub covered_lines: usize,
}

impl SourceFileInfo{
    pub fn new(src: &SourceFile) -> Self{
        let path = src.path_string();
        let base_name = src.path_string();
        let lines = src.reachable_lines.len();
        let covered_lines = src.covered_lines.len();
        return Self{base_name, path, lines, covered_lines};
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SourceFile {
    id: SrcID,
    path: MappedFilename,
    reachable_lines: HashSet<LineNum>,
    covered_lines: HashSet<LineNum>,
}

impl SourceFile {
    pub fn new(id: SrcID, path: MappedFilename) -> Self {
        return Self {
            id,
            path,
            reachable_lines: HashSet::new(),
            covered_lines: HashSet::new(),
        };
    }

    pub fn add_covered_line(&mut self, line: LineNum) {
        self.covered_lines.insert(line);
    }

    pub fn add_reachable_line(&mut self, line: LineNum) {
        self.reachable_lines.insert(line);
    }

    pub fn reachable_lines(&self) -> &HashSet<LineNum> {
        return &self.reachable_lines;
    }

    pub fn is_reachable(&self, l: LineNum) -> bool {
        return self.reachable_lines.contains(&l);
    }

    pub fn path_string(&self) -> String {
        return match &self.path {
            MappedFilename::Known(path) => path.to_string(),
            MappedFilename::Unknown(path) => path.to_string(),
        }
    }

    pub fn file_name(&self) -> String{
        return Path::new(&self.path_string()).file_name().expect("source file should be a file").to_str().unwrap().to_string();
    }

    //pub fn id(&self) -> SrcID {
    //    return self.id;
    //}

    pub fn path(&self) -> &MappedFilename {
        return &self.path;
    }

    pub fn find_matches(&self, pattern: &Regex) -> Vec<LineID> {
        let mut res = vec!();
        if let MappedFilename::Known(ref path) = &self.path{
        if let Ok(file) = File::open(path){
            let reader = BufReader::new(file);
                for (i,line) in reader.lines().enumerate() {
                    if let Ok(l) = line {
                        if pattern.is_match(&l) {
                            res.push(LineID::new(self.id, LineNum::new(i+1)));
                        }
                    }
                }
            }
        }

        return res;
    }
}
