use super::*;
use pretty_assertions::assert_eq;

#[track_caller]
fn assert_input_output(input: &str, output: impl IntoIterator<Item = TokenValue>) {
    let scanner = Lexer::new(input, None);
    let tokens: Result<Vec<TokenValue>, NovaError> = scanner.map(|t| t.map(|t| t.value)).collect();
    let tokens = tokens.expect("Lexing failied unexpectedly");
    assert_eq!(tokens, output.into_iter().collect::<Vec<_>>())
}

use crate::Operator::*;
use crate::StructuralSymbol::*;
use TokenValue::*;
#[test]
fn int_parsing() {
    assert_input_output("1 2 3", [Integer(1), Integer(2), Integer(3)]);
}

#[test]
fn comments() {
    assert_input_output(
        "1// 2 3
                4/* 5 6 7*/
                8",
        [Integer(1), Integer(4), Integer(8)],
    );
}

#[test]
fn int_fn_call() {
    assert_input_output(
        "123.foo()",
        [
            Integer(123),
            StructuralSymbol(Dot),
            Identifier("foo".to_string()),
            StructuralSymbol(LeftParen),
            StructuralSymbol(RightParen),
        ],
    );
}

#[test]
fn float_fn_call() {
    assert_input_output(
        "1.2.foo()",
        [
            Float(1.2),
            StructuralSymbol(Dot),
            Identifier("foo".to_string()),
            StructuralSymbol(LeftParen),
            StructuralSymbol(RightParen),
        ],
    );
}

#[test]
fn short_float() {
    assert_input_output(
        ".1 1. 2. 1.foo 1.2",
        [
            Float(0.1),
            Float(1.),
            Float(2.),
            Integer(1),
            StructuralSymbol(Dot),
            Identifier("foo".to_string()),
            Float(1.2),
        ],
    );
}

#[test]
fn escaped_string() {
    assert_input_output(
        r#" "hello\nworld!\"\0\r\t" "#,
        [StringLiteral("hello\nworld!\"\0\r\t".to_string())],
    );
}

#[test]
fn raw_string() {
    assert_input_output(
        r#####" r###"hello\nworld!\"\0\r\t"### "#####,
        [StringLiteral(r###"hello\nworld!\"\0\r\t"###.to_string())],
    );
}

#[test]
fn escaped_chars() {
    assert_input_output(
        "'a' 'b' '\n' '\''",
        [Char('a'), Char('b'), Char('\n'), Char('\'')],
    )
}

#[test]
fn non_decimal_ints() {
    assert_input_output(
        "0b1 0b1010 0b111111111111111111111111111111111111111111111111111111111111111
        0o1 0o12 0o777777777777777777777
        0x1 0xa 0x7fffffffffffffff
        ",
        [Integer(1), Integer(10), Integer(i64::MAX)]
            .iter()
            .cycle()
            .take(9)
            .cloned(),
    )
}

#[test]
fn idents() {
    assert_input_output(
        "abc __aabb _123 ____",
        ["abc", "__aabb", "_123", "____"].map(|i| Identifier(i.to_string())),
    )
}

#[test]
fn ranges() {
    assert_input_output("1..2", [Integer(1), Operator(ExclusiveRange), Integer(2)])
}

#[test]
fn operators() {
    assert_input_output(
        "+= -= && || :: : >= <= == = -> <- > < + - / * % != ! ~> ..= ..",
        [
            AddAssign,
            SubAssign,
            And,
            Or,
            DoubleColon,
            Colon,
            GreaterOrEqual,
            LessOrEqual,
            Equal,
            Assignment,
            RightArrow,
            LeftArrow,
            Greater,
            Less,
            Addition,
            Subtraction,
            Division,
            Multiplication,
            Modulo,
            NotEqual,
            Not,
            RightTilde,
            InclusiveRange,
            ExclusiveRange,
        ]
        .map(Operator),
    )
}

#[test]
fn structural_symbols() {
    assert_input_output(
        "; () [ ] , { } $ @ ? # . & ~ |",
        [
            Semicolon,
            LeftParen,
            RightParen,
            LeftSquareBracket,
            RightSquareBracket,
            Comma,
            LeftBrace,
            RightBrace,
            DollarSign,
            At,
            QuestionMark,
            Pound,
            Dot,
            Ampersand,
            Tilde,
            Pipe,
        ]
        .map(StructuralSymbol),
    )
}

#[test]
fn keywords() {
    assert_input_output(
        "true false in",
        [
            Bool(true),
            Bool(false),
            Keyword(common::tokens::KeyWord::In),
        ],
    );
}
