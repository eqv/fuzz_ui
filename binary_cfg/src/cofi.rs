#[derive(PartialEq, Eq, Debug)]
pub struct Cofi{
    pub addr: u64,
    pub size: u64,
    pub ctype: CofiType
}

impl Cofi{

    pub fn new(addr: u64, size:u64, ctype: CofiType) -> Self{
        return Cofi{addr, size, ctype}
    }

    pub fn invalid(addr: u64) -> Self{
        return Cofi{addr, size: 0, ctype: CofiType::Invalid()};
    }
    pub fn fallthrough(&self) -> u64{
        self.addr+self.size
    }

    pub fn get_successors(&self) -> Vec<u64>{
        match self.ctype{
            CofiType::JmpCond(a) => vec!(self.fallthrough(),a),
            CofiType::JmpCondIndirect() => vec!(self.fallthrough()),
            CofiType::JmpIndirect() => vec!(),
            CofiType::Jmp(a) => vec!(a),
            CofiType::Ret() => vec!(),
            CofiType::Call(a) => vec!(self.fallthrough(), a),
            CofiType::CallIndirect() => vec!(self.fallthrough()),
            CofiType::Invalid() => vec!(),
        }
    }

}

#[derive(PartialEq, Eq, Debug)]
pub enum CofiType{
    JmpCond(u64),
    JmpCondIndirect(),
    JmpIndirect(),
    Jmp(u64),
    Ret(),
    Call(u64),
    CallIndirect(),
    Invalid()
}

impl CofiType{
    pub fn is_indirect(&self) -> bool {
        match self {
            &CofiType::JmpCond(_) => false,
            &CofiType::JmpCondIndirect() => true,
            &CofiType::JmpIndirect() => true,
            &CofiType::Jmp(_) => false,
            &CofiType::Ret() => true,
            &CofiType::Call(_) => false,
            &CofiType::CallIndirect() => true,
            &CofiType::Invalid() => false,
        }
    }

    pub fn is_conditional(&self) -> bool {
        match self {
            &CofiType::JmpCond(_) => true,
            &CofiType::JmpCondIndirect() => true,
            _ => false
        }
    }

    pub fn next_addr(&self) -> Option<u64> {
        match self {
            &CofiType::Jmp(a) => return Some(a),
            &CofiType::Call(a) => return Some(a),
            _ => return None
        }
    }

}
