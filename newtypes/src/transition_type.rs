#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize)]
pub enum TransitionType {
    OutOfTrace,
    Taken,
    NotTaken,
    Indirect,
    Direct,
    Call,
    Ret,
    Exit,
    Entry,
    Unknown,
}

impl TransitionType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => return Some(TransitionType::Taken),
            1 => return Some(TransitionType::NotTaken),
            2 => return Some(TransitionType::Direct),
            3 => return Some(TransitionType::Ret),
            4 => return Some(TransitionType::Indirect),
            5 => return Some(TransitionType::Exit),
            6 => return Some(TransitionType::Entry),
            7 => return Some(TransitionType::OutOfTrace),
            8 => return Some(TransitionType::Unknown),
            _ => return None,
        }
    }
}
