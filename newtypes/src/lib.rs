extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate snafu;
extern crate nix;
extern crate ron;
extern crate clap;

use std::cmp::Ordering;

mod transition_type;
mod config;
pub mod error;
pub use config::*;
pub use transition_type::TransitionType;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub struct SrcID(usize);
impl SrcID {
    pub fn new(id: usize) -> Self {
        return SrcID(id);
    }
    pub fn as_usize(&self) -> usize {
        return self.0;
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub enum IsXref {
    Xref,
    Direct,
}

impl IsXref{
    pub fn is_xref(&self) -> bool {
        return self == &IsXref::Xref;
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub struct LineNum(usize);
impl LineNum {
    pub fn new(id: usize) -> Self {
        return LineNum(id);
    }
    pub fn as_usize(&self) -> usize {
        return self.0;
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub struct InputID(usize);
impl InputID {
    pub fn new(id: usize) -> Self {
        return InputID(id);
    }
    pub fn as_usize(&self) -> usize {
        return self.0;
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub struct LineID {
    file: SrcID,
    num: LineNum,
}

impl LineID {
    pub fn new(file: SrcID, num: LineNum) -> Self {
        return Self { file, num };
    }
    pub fn new_i(file: SrcID, num: usize) -> Self {
        return Self {
            file,
            num: LineNum(num),
        };
    }
    pub fn num(&self) -> LineNum {
        return self.num;
    } 

    pub fn file(&self) -> SrcID {
        return self.file;
    }
}

impl Ord for LineID {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.file.as_usize().cmp(&other.file.as_usize())
        .then(self.num.as_usize().cmp(&other.num.as_usize()));
    }
}

impl PartialOrd for LineID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}



#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub struct Transition{
    pub src: LineID,
    pub kind: TransitionType,
    pub dst: LineID,
}

impl Transition {
    pub fn new(src: LineID, kind: TransitionType, dst: LineID) -> Self{ return Self{src, kind, dst}; }
    pub fn call(src: LineID, dst: LineID)      -> Self { return Self{src, kind:TransitionType::Call,       dst } }
    pub fn oob(src: LineID, dst: LineID)       -> Self { return Self{src, kind:TransitionType::OutOfTrace, dst } }
    pub fn ret(src: LineID, dst: LineID)       -> Self { return Self{src, kind:TransitionType::Ret,        dst } }
    pub fn taken(src: LineID, dst: LineID)     -> Self { return Self{src, kind:TransitionType::Taken,      dst } }
    pub fn not_taken(src: LineID, dst: LineID) -> Self { return Self{src, kind:TransitionType::NotTaken,   dst } }
    pub fn indirect(src: LineID, dst: LineID)  -> Self { return Self{src, kind:TransitionType::Indirect,   dst } }
}



#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct CoverageCounts {
    pub num_a: usize,
    pub num_b: usize,
}

impl CoverageCounts {
    pub fn new() -> Self{ return Self{num_a: 0, num_b:0}}
    pub fn css_class(&self) -> &'static str {
        match (self.num_a > 0, self.num_b > 0) {
            (false, false) => "neither",
            (true, true) => "both",
            (true, false) => "seta",
            (false, true) => "setb",
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
