
use newtypes::TransitionType;

//In a Raw trace, the src can point to either the address of the first (Qemu traces) or the address of the cofi instruction (kAFL/IntelPT).
//The transaction type is optional and will be set automatically by the disassembler.

#[derive(Debug)]
pub struct RawTrace{
    pub transitions: Vec<BinaryTransition>
}

impl RawTrace {
    pub fn new() -> Self {
        return Self{transitions: vec!()};
    }

    pub fn add_transition(&mut self, trans: BinaryTransition){
        self.transitions.push(trans);
    }
}


//In contrast to a RawTranstion, a BinaryTansition always contains the addres of the source cofi instruction. 
//Additionally, the TransitionType will be filled with values.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BinaryTransition{
    pub src: u64,
    pub dst: u64,
    pub cnt: u64,
    pub t_type: TransitionType,
}

impl BinaryTransition {
    pub fn new(src: u64, dst: u64, t_type: TransitionType, cnt: u64) -> Self{
        return BinaryTransition{src,dst, t_type, cnt};
    }
}

#[derive(Debug)]
pub struct BinaryTrace{
    pub transitions: Vec<BinaryTransition>
}

impl BinaryTrace {
    pub fn new() -> Self {
        return Self{transitions: vec!()};
    }

    pub fn add_transition(&mut self, trans: BinaryTransition){
        self.transitions.push(trans);
    }
}
