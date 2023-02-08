SourceMapResult{
}

InputResult {
    data
}

TraceResult {
    input
    data
}

LintWarnings {
    query..
}

DBGTrace {
    input,
    query,
    data,
}

struct RawResultDB{
    sources: HashMap<SrcID, Arc<SourceMapResult>>,
    inputs: HashMap<InputID, Arc<InputResult>>,
    traces: HashMap<TraceID, Arc<TraceResult>>,
    lint_warnings: HashMap<LintID, Arc<LintWarnings>>,
    dbg_infos: HashMap<DbgID, Arc<DBGTrace>>
}

FilteredResults{
}

struct AgregateResults{
    CFG
    SourceTransitions
}
