# uscan
A universal source code scanner

/!\ work-in-progress

# features
* configurable keywords, symbols and comments
* handles nested multi-line comments
* handles decimal (15), hexadecimal (0xf or 0xF) and binary (0b1111) literal numbers

# usage

```rust
const LUA_CONFIG: ScannerConfig = ScannerConfig {
    keywords: &[
        "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in",
        "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
    ],
    symbols: &[
        "...", "..", "==", "~=", "<=", ">=", "+", "-", "*", "/", "%", "^", "#", "<", ">", "=", "(",
        ")", "{", "}", "[", "]", ";", ":", ",", ".",
    ],
    single_line_cmt: Some("--"),
    multi_line_cmt_start: Some("--[["),
    multi_line_cmt_end: Some("]]"),
};

let mut scanner_data = ScannerData::default();
let mut scanner = Scanner::default();
scanner.run(source_code, &LUA_CONFIG, &mut scanner_data)?;
```

=> you can now use the ScannerData struct in your parser to build your AST :

```rust
pub enum TokenType {
    Symbol(String),
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String, Number),
    Keyword(String),
    Comment(String),
    // space
    Ignore,
    NewLine,
    Eof,
    Unknown,
}

pub struct ScannerData {
    /// complete source code
    pub source: Vec<char>,
    /// resulting list of tokens
    pub token_types: Vec<TokenType>,
    /// token start line in the source code
    pub token_lines: Vec<usize>,
    /// token start offset from its line beginning
    pub token_start: Vec<usize>,
    /// token length in characters
    /// not always = token value's length.
    /// for example for TokenType::StringLiteral("aa") the value length is 2 but the token length including the quotes is 4
    pub token_len: Vec<usize>,
}
```