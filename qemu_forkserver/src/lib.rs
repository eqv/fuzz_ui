extern crate tempfile;
extern crate nix;
extern crate byteorder;
extern crate snafu;

use nix::unistd;
use std::os::unix::io::RawFd;
use nix::unistd::{fork, ForkResult};
use nix::fcntl;
use nix::sys::stat;
use std::os::unix::io::AsRawFd;
use std::ffi::CString;
use nix::unistd::Pid;
use nix::sys::signal::{self, Signal};

use std::io::{BufReader};
use timeout_readwrite::TimeoutReader;
use std::time::Duration;

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::os::unix::io::FromRawFd;

use snafu::{ResultExt};
use newtypes::error::*;
use std::{time,thread};

pub struct ForkServer{
    inp_file: File,
    ctl_in : File,
    st_out : std::io::BufReader<TimeoutReader<File>>,
}

impl ForkServer{
    pub fn new(path: String, args: Vec<String>, hide_output: bool, outdir: String) -> Self {
        let inp_file = tempfile::NamedTempFile::new().expect("couldn't create temp file");
        let (inp_file, in_path) = inp_file.keep().expect("couldn't persists temp file for input");
        let inp_file_path = in_path
            .to_str()
            .expect("temp path should be unicode!")
            .to_string();
        let args = args.into_iter().map(|s| if s == "@@" { inp_file_path.clone() } else { s }).collect::<Vec<_>>();
        let (ctl_out, ctl_in) = nix::unistd::pipe().expect("failed to create ctl_pipe");
        let (st_out, st_in) = nix::unistd::pipe().expect("failed to create st_pipe");

        //let ten_millis = time::Duration::from_millis(10000);
        //thread::sleep(ten_millis);

        match fork().expect("couldn't fork") {
            // Parent returns
            ForkResult::Parent { child, .. } => {
                unistd::close(ctl_out).expect("coulnd't close ctl_out");
                unistd::close(st_in).expect("coulnd't close st_out");;
                let mut st_out = BufReader::new(TimeoutReader::new(unsafe { File::from_raw_fd(st_out) }, Duration::new(1, 0)));
                st_out.read_u32::<LittleEndian>().expect("couldn't read child hello");
                return Self{inp_file: inp_file, ctl_in: unsafe{ File::from_raw_fd(ctl_in) }, st_out};
            },
            //Child does complex stuff
            ForkResult::Child => {
                let forkserver_fd = 198; // from AFL config.h
                unistd::dup2(ctl_out, forkserver_fd as RawFd).expect("couldn't dup2 ctl_our to FROKSRV_FD");
                unistd::dup2(st_in, (forkserver_fd+1) as RawFd).expect("couldn't dup2 ctl_our to FROKSRV_FD+1");;

                unistd::dup2(inp_file.as_raw_fd(),0).expect("couldn't dup2 input file to stdin");
                unistd::close(inp_file.as_raw_fd()).expect("couldn't close input file");

                unistd::close(ctl_in).expect("couldn't close ctl_in");
                unistd::close(ctl_out).expect("couldn't close ctl_out");
                unistd::close(st_in).expect("couldn't close ctl_out");
                unistd::close(st_out).expect("couldn't close ctl_out");
            
                let path = CString::new(path).expect("binary path must not contain zero");
                let args = args.into_iter().map(|s| CString::new(s).expect("args must not contain zero")).collect::<Vec<_>>();
                let trace_dir =CString::new(format!("TRACE_OUT_DIR={}",outdir)).expect("outdir must not ocntain zero");
                let qemu_log = CString::new("QEMU_LOG=nochain").unwrap();
                let env = vec!( trace_dir , qemu_log );

                if false && hide_output {
                    let null = fcntl::open("/dev/null",fcntl::OFlag::O_RDWR, stat::Mode::empty()).expect("couldn't open /dev/null");
                    unistd::dup2(null, 1 as RawFd).expect("couldn't dup2 /dev/null to stdout");
                    unistd::dup2(null, 2 as RawFd).expect("couldn't dup2 /dev/null to stderr");
                    unistd::close(null).expect("couldn't close /dev/null");
                }
                println!("EXECVE  {:?} {:?} {:?}",path,args,env);
                unistd::execve(&path, &args, &env).expect("couldn't execve afl-qemu-tarce");
                unreachable!();
            }
        }
    }

    pub fn run(&mut self, data: &[u8]) -> Result<u32,Error>{
        unistd::ftruncate(self.inp_file.as_raw_fd(), 0).context(QemuRunNix{task: "Couldn't truncate inp_file"})?;
        unistd::lseek(self.inp_file.as_raw_fd(), 0, unistd::Whence::SeekSet).context(QemuRunNix{task: "Couldn't seek inp_file"})?;
        unistd::write(self.inp_file.as_raw_fd(), data).context(QemuRunNix{task: "Couldn't write data to inp_file"})?;
        unistd::lseek(self.inp_file.as_raw_fd(), 0, unistd::Whence::SeekSet).context(QemuRunNix{task: "Couldn't seek inp_file"})?; 

        unistd::write(self.ctl_in.as_raw_fd(), &[0,0,0,0]).context(QemuRunNix{task: "Couldn't send start command"})?;
        
        let pid = Pid::from_raw(self.st_out.read_i32::<LittleEndian>().context(QemuRunIO{task: "Couldn't read target pid"})?);

        if let Ok(status) = self.st_out.read_u32::<LittleEndian>(){
            return Ok(status);
        }
        signal::kill(pid, Signal::SIGKILL).context(QemuRunNix{task: "Couldn't kill timed out process"})?;
        self.st_out.read_u32::<LittleEndian>().context(QemuRunIO{task: "couldn't read timeout exitcode"})?;
        return Ok(0);
    }
}


#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn run_forkserver() {
        let hide_output = false;
        let target = "../afl_fast_bin_cov/afl-qemu-trace_64".to_string();
        let args =  vec!("afl-qemu-trace_64".to_string(),"../afl_fast_bin_cov/workdir/test_sleep".to_string());
        let outdir = "/tmp/".to_string();
        let mut fork = ForkServer::new(target, args, hide_output, outdir); 
        fork.run(&vec!(1,2,3,4,5,6,20));
    }
}
