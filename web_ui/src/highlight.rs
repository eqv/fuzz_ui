use std::io::BufRead;
use syntect::easy::HighlightFile;
use syntect::highlighting::{Style, ThemeSet};
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;

pub fn get_hl_file_content(path: &str) -> Vec<String> {
    let mut res = vec![];
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["InspiredGitHub"];

    let mut highlighter = HighlightFile::new(path, &ss, &theme).unwrap();

    // We use read_line instead of `for line in highlighter.reader.lines()` because that
    // doesn't return strings with a `\n`, and including the `\n` gets us more robust highlighting.
    // See the documentation for `SyntaxSetBuilder::add_from_folder`.
    // It also allows re-using the line buffer, which should be a tiny bit faster.
    let mut line = String::new();
    while highlighter.reader.read_line(&mut line).unwrap() > 0 {
        {
            let regions: Vec<(Style, &str)> = highlighter.highlight_lines.highlight(&line, &ss);
            res.push(styled_line_to_highlighted_html(
                &regions[..],
                IncludeBackground::No,
            ));
        }
        line.clear();
    }
    return res;
}
