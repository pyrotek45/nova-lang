#![no_main]
use libfuzzer_sys::fuzz_target;

// Fuzz the Nova parser: feed arbitrary source text through the full
// lex → parse pipeline. Neither phase must ever panic.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Lex — if it errors that's fine, just don't panic
        let mut lex = lexer::Lexer::new(s, None);
        let tokens = match lex.tokenize() {
            Ok(t) => t,
            Err(_) => return,
        };

        // Parse — errors are expected and valid; panics are bugs
        let mut p = parser::default();
        p.input = tokens;
        let _ = p.parse();
    }
});
