extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate regex;

extern crate newtypes;
extern crate dump_srcmap;
extern crate binary_cfg;

mod source_state;
mod source_info;
mod source_trace;

pub use crate::source_info::SourceInfo;
pub use crate::source_trace::SourceTrace;
pub use crate::source_state::{SourceFileInfo,SourceFile};


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
