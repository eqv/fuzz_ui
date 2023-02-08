extern crate serde_derive;
extern crate newtypes;
extern crate capstone;
extern crate theban_interval_tree;
extern crate memrange;
extern crate syntect;

mod cfg;
mod cofi;
mod target;
mod binary_trace;
mod disassembler;


pub use cfg::{BB,CFG};
pub use binary_trace::{BinaryTrace,BinaryTransition, RawTrace};
pub use cofi::{Cofi,CofiType};
pub use target::TargetProg;
pub use disassembler::Disassembler;


#[cfg(test)]
mod tests {
    use crate::*;
    
    #[test]
    fn it_works() {
        let _tr = BinaryTrace::new();
        //let cnt = 1;
        //tr.add_transition(0xffffffff_ffffffff,0, TransitionType::Unknown, cnt);
        //tr.add_transition(0, 10, TransitionType::Unknown, cnt);
        //tr.add_transition(10, 3, TransitionType::Unknown, cnt);
        //tr.add_transition(5, 6, TransitionType::Unknown, cnt);
        //tr.add_transition(5, 8, TransitionType::Unknown, cnt);
        //tr.add_transition(9, 0xffffffff_ffffffff, TransitionType::Unknown, cnt);
    }
}
