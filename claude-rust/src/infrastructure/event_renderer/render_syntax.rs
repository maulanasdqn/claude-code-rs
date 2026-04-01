use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

static SS: OnceLock<SyntaxSet> = OnceLock::new();
static TS: OnceLock<ThemeSet> = OnceLock::new();

fn ss() -> &'static SyntaxSet {
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn ts() -> &'static ThemeSet {
    TS.get_or_init(ThemeSet::load_defaults)
}

pub struct Highlighter<'a> {
    inner: HighlightLines<'a>,
}

pub fn new_highlighter(lang: &str) -> Option<Highlighter<'static>> {
    let syntax = ss()
        .find_syntax_by_token(lang)
        .or_else(|| ss().find_syntax_by_extension(lang))?;
    let theme = ts().themes.get("base16-eighties.dark").or_else(|| ts().themes.values().next())?;
    let inner = HighlightLines::new(syntax, theme);
    Some(Highlighter { inner })
}

pub fn highlight_line(h: &mut Highlighter<'_>, line: &str) -> String {
    match h.inner.highlight_line(line, ss()) {
        Ok(ranges) => {
            let out = as_24_bit_terminal_escaped(&ranges, false);
            format!("{out}\x1b[0m")
        }
        Err(_) => line.to_string(),
    }
}
