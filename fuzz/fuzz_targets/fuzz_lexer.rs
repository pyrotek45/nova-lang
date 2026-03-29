#![no_main]
use libfuzzer_sys::fuzz_target;

// Fuzz the Nova lexer: feed arbitrary bytes as source code.
// The lexer must NEVER panic — it should always either succeed or return an error.
fuzz_target!(|data: &[u8]| {
    // Only test valid UTF-8 input (the lexer expects text)
    if let Ok(s) = std::str::from_utf8(data) {
        // Lexer::new takes a source string; tokenize() is where work happens.
        let mut lex = lexer::Lexer::new(s, None);
        // Must not panic; errors are expected and fine.
        let _ = lex.tokenize();
    }
});
