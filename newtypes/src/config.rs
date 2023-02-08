use clap::{value_t, App, Arg};
use std::fs::File;
use crate::ron;

#[derive(Debug,Serialize,Deserialize)]
pub enum TargetMachine{
    X86_32,
    X86_64,
}

impl TargetMachine{
    pub fn bits(&self) -> usize{
        return match self {
            TargetMachine::X86_32 => 32,
            TargetMachine::X86_64 => 64,
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct TargetConfig{
    pub path: String,
    pub args: Vec<String>,
    pub pie_base: Option<u64>,
    pub machine: TargetMachine,
}

impl TargetConfig{
    pub fn new_x86_32(path: String, args: Vec<String>, pie_base: Option<u64>) -> Self{
        let machine = TargetMachine::X86_32;
        return Self{path, args, machine, pie_base};
    }
        pub fn new_x86_64(path: String, args: Vec<String>, pie_base: Option<u64>) -> Self{
        let machine = TargetMachine::X86_64;
        return Self{path, args, machine, pie_base};
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Dirs{
    pub ijon_base_dir: String,
    pub source_dir: String,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Config {
    pub dirs: Dirs,
    pub target: TargetConfig,
}

impl Config{
    pub fn from_args() -> Self{
        let matches = App::new("IjonWebUI")
        .about("Better Fuzzing Coverage")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG_PATH")
                .takes_value(true)
                .help("path to the config.ron, default: ./config.ron"),
        )
        //.arg(
        //    Arg::with_name("workdir")
        //        .short("w")
        //        .long("workdir")
        //        .value_name("WORKDIR_PATH")
        //        .takes_value(true)
        //        .help("overrides the workdir path in the config"),
        //)
        //.arg(
        //    Arg::with_name("cpu_start")
        //        .short("p")
        //        .long("cpu")
        //        .value_name("CPU_START")
        //        .takes_value(true)
        //        .help("overrides the config value for the first CPU to pin threads to"),
        //)
        //.arg(
        //    Arg::with_name("seed")
        //        .long("seed")
        //        .value_name("SEED")
        //        .takes_value(true)
        //        .help("runs the fuzzer with a specific seed, if not give, a seed is generated from a secure prng"),
        //)
        .get_matches();
    
    let config = matches
        .value_of("config")
        .unwrap_or("./config.ron")
        .to_string();

    let cfg_file = File::open(&config).unwrap();
    let cfg: Self = ron::de::from_reader(cfg_file).unwrap();
    
    //if let Some(path) = matches.value_of("workdir") {
    //    config.workdir_path = path.to_string();
    //}
    //if let Ok(start_cpu_id) = value_t!(matches, "cpu_start", usize) {
    //    config.cpu_pin_start_at = start_cpu_id;
    //}
    return cfg;
    }
}