use snafu::{Snafu, Backtrace};

use std::path::PathBuf;



#[derive(Debug, Snafu)]
#[snafu(visibility="pub")]
pub enum Error {
    #[snafu(display("Could not handle qemu trace file to {} {}", path.display(), source))]
    ReadQemuTrace { path: PathBuf, source: std::io::Error },

    #[snafu(display("Could not parse integer in {} {}", line, source))]
    ParseIntQemuTrace {line: String, source: std::num::ParseIntError },
    
    #[snafu(display("Could not parse line {}", line))]
    ParseLineQemuTrace { line: String, backtrace: Backtrace },

    #[snafu(display("Qemu did not produce any output"))]
    NoQemuOutput {backtrace: Backtrace },

    #[snafu(display("Could not communicate with QemuForkserver {} {} ", task, source))]
    QemuRunNix { task: String, source: nix::Error },

    #[snafu(display("Could not communicate with QemuForkserver {} {} ", task, source))]
    QemuRunIO { task: String, source: std::io::Error },

    #[snafu(display("Failed to disassemblr {}", task))]
    DisassemblyError {task: String, backtrace: Backtrace },
}