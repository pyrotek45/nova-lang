#![no_main]
use libfuzzer_sys::fuzz_target;

// Fuzz the full Nova compilation pipeline (lex → parse).
// The pipeline must never panic — only return errors.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Run through the lexer
        let mut lex = lexer::Lexer::new(s, None);
        let tokens = match lex.tokenize() {
            Ok(t) => t,
            Err(_) => return,
        };

        // Run through the parser
        let mut p = parser::default();
        p.input = tokens;
        let _ = p.parse();
        // Compiler stage fuzzing would go here once we have a no-file-path
        // compiler entry point.
    }
});
