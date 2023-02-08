extern crate gimli;
extern crate object;
extern crate memmap;
extern crate glob;
extern crate newtypes;

use std::path::Path;
use crate::object::{Object, ObjectSection};
use std::{borrow, fs, path};
use std::collections::HashMap;

use newtypes::{SrcID, LineNum};

pub type StartAddr = u64;
pub type EndAddr = u64;

#[derive(Debug,PartialEq,Eq,Clone)]
pub enum MappedFilename{
    Known(String),
    Unknown(String),
}


pub fn get_line_info(path: &str, pie_base: Option<u64>) -> (HashMap<SrcID, String>, Vec<(StartAddr,EndAddr, SrcID, LineNum)>) {
    let file = fs::File::open(&path).unwrap();
    let mmap = unsafe { memmap::Mmap::map(&file).unwrap() };
    let object = object::File::parse(&*mmap).unwrap();
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };
    return dump_file(pie_base, &object, endian);
}

pub fn dump_file(pie_base: Option<u64>, object: &object::File, endian: gimli::RunTimeEndian) -> (HashMap<SrcID, String>, Vec<(StartAddr,EndAddr, SrcID, LineNum)>) {
    let mut filename_to_srcid = HashMap::new();
    let mut srcid_to_filename = HashMap::<SrcID,String>::new();
    let mut line_data = Vec::<(StartAddr,EndAddr,SrcID, LineNum)>::new();
    let base = pie_base.unwrap_or(0);

    // Load a section and return as `Cow<[u8]>`.
    let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>, gimli::Error> {
        match object.section_by_name(id.name()) {
            Some(ref section) => {
                Ok(section.uncompressed_data().unwrap_or(borrow::Cow::Borrowed(&[][..])))
            }  
            None => Ok(borrow::Cow::Borrowed(&[][..])),
        }   
    };
    // Load a supplementary section. We don't have a supplementary object file,
    // so always return an empty slice.
    let load_section_sup = |_| Ok(borrow::Cow::Borrowed(&[][..]));

    // Load all of the sections.
    let dwarf_cow = gimli::Dwarf::load(&load_section, &load_section_sup).unwrap();

    // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
    let borrow_section: &dyn for<'a> Fn(
        &'a borrow::Cow<[u8]>,
    ) -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
        &|section| gimli::EndianSlice::new(&*section, endian);

    // Create `EndianSlice`s for all of the sections.
    let dwarf = dwarf_cow.borrow(&borrow_section);

    // Iterate over the compilation units.
    let mut iter = dwarf.units();
    while let Some(header) = iter.next().unwrap() {
        let unit = dwarf.unit(header).unwrap();

        // Get the line program for the compilation unit.
        if let Some(program) = unit.line_program.clone() {
            let comp_dir = if let Some(ref dir) = unit.comp_dir {
                path::PathBuf::from(dir.to_string_lossy().into_owned())
            } else {
                path::PathBuf::new()
            };

            // Iterate over the line program rows.
            let mut rows = program.rows();
            let mut last_info = None;
            while let Some((header, row)) = rows.next_row().unwrap() {
                if row.end_sequence() {
                    // End of sequence indicates a possible gap in addresses.
                    if let Some((addr ,src,line)) = last_info{
                        line_data.push((base+addr,base+row.address(), src, line));
                    }
                } else {
                    // Determine the path. Real applications should cache this for performance.
                    let mut path = path::PathBuf::new();
                    if let Some(file) = row.file(header) {
                        path = comp_dir.clone();
                        if let Some(dir) = file.directory(header) {
                            path.push(dwarf.attr_string(&unit, dir).unwrap().to_string_lossy().as_ref());
                        }
                        path.push(
                            dwarf
                                .attr_string(&unit, file.path_name()).unwrap()
                                .to_string_lossy()
                                .as_ref(),
                        );
                    }
                    let path = path.to_string_lossy().to_string();
                    if !filename_to_srcid.contains_key(&path){
                        let newid = SrcID::new(filename_to_srcid.len()+1);
                        filename_to_srcid.insert(path.clone(), newid);
                        srcid_to_filename.insert(newid, path.clone());
                    }
                    let srcid = filename_to_srcid.get(&path).unwrap();
                    let line = LineNum::new(row.line().unwrap_or(0) as usize);

                    if let Some((addr, src,line)) = last_info.take(){
                        line_data.push((base+addr,base+row.address(), src, line));
                    }
                    last_info = Some((row.address(), *srcid, line));
                }
            }
        }
    }
    return (srcid_to_filename, line_data);
}

fn get_filename_to_paths_in_src_dir(src_dir: &str) -> HashMap<String, Vec<String>> {
    let mut filename_to_paths = HashMap::new();
    let pat = src_dir.to_string()+"/**/*";
    for path in glob::glob(&pat).expect("unable to parse glob pattern for src files"){
        if let Ok(path) = path{
            if let Some(name) = path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string()){
                if let Some(path) = path.to_str(){
                    if !filename_to_paths.contains_key(&name){
                        filename_to_paths.insert(name.clone(), vec!());
                    }
                    filename_to_paths.get_mut(&name).unwrap().push(path.to_string());
                }
            }
        }
    }
    return filename_to_paths
}

fn get_common_postfix(a: &str, b:&str) -> usize{
    return a.chars().rev().zip(b.chars().rev()).take_while(|(a,b)| a==b).count();
}



pub fn find_best_src_files(src_files: HashMap<SrcID, String>, src_dir: &str) -> HashMap<SrcID, MappedFilename> {
    let filename_to_paths = get_filename_to_paths_in_src_dir(src_dir);
    let mut res = HashMap::new();
    for (id, path_str) in src_files {
        let path = Path::new(&path_str);
        if let Some(basename) = path.file_name().and_then(|s| s.to_str()) {
            if let Some(options) = filename_to_paths.get(basename) {
                if let Some(matching_path) = options.iter().max_by_key(|s| get_common_postfix(s, &path_str)) {
                    res.insert(id, MappedFilename::Known(matching_path.to_string()));
                    continue
                }
            }
        }
        res.insert(id, MappedFilename::Unknown(path_str.to_string()));
        println!("Warning: couldn't match src file {}", path_str);
    }
    return res;
}

