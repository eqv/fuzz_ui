use std::collections::{HashMap};
use crate::newtypes::{LineID, IsXref, TransitionType, CoverageCounts};

#[derive(Hash, Eq, PartialEq, Clone, Debug, Serialize)]
pub struct JsonTransitionInfo{
    is_xref: bool,
    css: String,
    line: LineID,
}

pub struct LineTransitions{
    transitions: HashMap<(TransitionType,LineID,IsXref), CoverageCounts>
}

impl LineTransitions{
    pub fn to_json(&self) -> HashMap<TransitionType, Vec<JsonTransitionInfo>> {
        let mut res = HashMap::new();
        for ((kind, line, xref), cov) in self.transitions.iter() {
            let val = res.entry(kind.clone()).or_insert_with(|| vec!());
            val.push( JsonTransitionInfo{is_xref: xref.is_xref(), css: cov.css_class().to_string(), line: *line});
        }
        return res
    }

    pub fn new() -> Self {
        let transitions = HashMap::new();
        return Self{transitions}
    }

    pub fn add_transition(&mut self, t: TransitionType, l: LineID, x: IsXref){
        self.transitions.entry((t, l, x)).or_insert_with(CoverageCounts::new).num_a += 1;
    }
}
