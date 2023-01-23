use std::io::Write;

pub type Number = f64;

/// The fields contain the line number and character position in the line
#[derive(Debug)]
pub enum ScanError {
    /// Unrecognized token.
    UnknownToken(usize, usize),
    /// Eof of file before the end of current token
    /// (for example, an unterminated string)
    UnexpectedEof(usize, usize),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, offset) = match self {
            ScanError::UnknownToken(line, offset) => (line, offset),
            ScanError::UnexpectedEof(line, offset) => (line, offset),
        };
        write!(
            f,
            "{}:{} : {}",
            line,
            offset,
            match self {
                ScanError::UnknownToken(_, _) => "unknown token",
                ScanError::UnexpectedEof(_, _) => "unexpected end of file",
            }
        )
    }
}

#[derive(Debug)]
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

impl TokenType {
    pub fn len(&self) -> usize {
        match self {
            TokenType::Symbol(s) => s.len(),
            TokenType::Identifier(s) => s.len(),
            TokenType::StringLiteral(s) => s.len() + 2,
            TokenType::Keyword(s) => s.len(),
            TokenType::NumberLiteral(s, _) => s.len(),
            TokenType::Comment(s) => s.len(),
            _ => 0,
        }
    }
}

#[derive(Default)]
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

impl ScannerData {
    pub fn dump(&self, out: &mut dyn Write) {
        for (i, token) in self.token_types.iter().enumerate() {
            writeln!(out, "[#{:03} line {}] {:?}", i, self.token_lines[i], *token).ok();
        }
    }
}

#[derive(Default)]
pub struct Scanner {
    // start of parsing position
    start: usize,
    // position during parsing of current token
    current: usize,
    // current line in file
    line: usize,
}

pub struct ScannerConfig {
    /// list of keywords, ordered by descending length
    pub keywords: &'static [&'static str],
    /// list of symbols, ordered by descending length
    pub symbols: &'static [&'static str],
    /// token starting a single line comment
    pub single_line_cmt: Option<&'static str>,
    /// token starting a multi line comment
    pub multi_line_cmt_start: Option<&'static str>,
    /// token ending a multi line comment
    pub multi_line_cmt_end: Option<&'static str>,
}

impl Scanner {
    /// scan the provided source code and return a list of tokens in the ScannerData structure
    pub fn run(
        &mut self,
        source: &str,
        config: &ScannerConfig,
        data: &mut ScannerData,
    ) -> Result<(), ScanError> {
        data.source = source.chars().collect();
        self.current = 0;
        self.line = 1;
        self.start = self.current;
        let mut exit = false;
        while !exit {
            let token = self.scan_token(data, config)?;
            match token {
                TokenType::Eof => exit = true,
                TokenType::Ignore => self.start = self.current,
                TokenType::NewLine => (),
                _ => self.add_token(token, data),
            }
        }
        Ok(())
    }
    fn add_token(&mut self, token: TokenType, data: &mut ScannerData) {
        let len = self.current - self.start;
        data.token_start.push(self.start);
        data.token_len.push(len);
        data.token_types.push(token);
        data.token_lines.push(self.line);
        self.start = self.current;
    }
    fn scan_token(
        &mut self,
        data: &mut ScannerData,
        config: &ScannerConfig,
    ) -> Result<TokenType, ScanError> {
        if self.current >= data.source.len() {
            return Ok(TokenType::Eof);
        }
        if let Some(token) = self.scan_comment(config, data) {
            return Ok(token);
        }
        if let Some(token) = self.scan_newline(data) {
            return Ok(token);
        }
        if let Some(token) = self.scan_space(data) {
            return Ok(token);
        }
        if let Some(token) = self.scan_symbol(data, config) {
            return Ok(token);
        }
        if let Some(token) = self.scan_keyword(data, config) {
            return Ok(token);
        }
        if let Some(token) = self.scan_string(data)? {
            return Ok(token);
        }
        if let Some(token) = self.scan_identifier(data) {
            return Ok(token);
        }
        if let Some(token) = self.scan_number(data) {
            return Ok(token);
        }
        data.token_len.push(1);
        data.token_start.push(self.current);
        data.token_types.push(TokenType::Unknown);
        data.token_lines.push(self.line);
        let token_id = data.token_len.len() - 1;
        Err(ScanError::UnknownToken(
            self.line,
            data.token_start[token_id],
        ))
    }
    fn scan_comment(
        &mut self,
        config: &ScannerConfig,
        data: &mut ScannerData,
    ) -> Option<TokenType> {
        if let Some(multi_start) = config.multi_line_cmt_start {
            if self.matches(multi_start, data) {
                if let Some(multi_end) = config.multi_line_cmt_end {
                    return self.scan_multi_line_comment(multi_start, multi_end, data);
                }
            }
        }
        if let Some(single_start) = config.single_line_cmt {
            if self.matches(single_start, data) {
                return self.scan_single_line_comment(data);
            }
        }
        None
    }
    fn scan_single_line_comment(&mut self, data: &mut ScannerData) -> Option<TokenType> {
        while self.current < data.source.len() && data.source[self.current] != '\n' {
            self.current += 1;
        }
        self.current += 1;
        self.line += 1;
        Some(TokenType::Comment(
            data.source[self.start..self.current - 1]
                .iter()
                .cloned()
                .collect::<String>(),
        ))
    }
    fn scan_multi_line_comment(
        &mut self,
        multi_start: &str,
        multi_end: &str,
        data: &mut ScannerData,
    ) -> Option<TokenType> {
        let mut level = 0;
        let mut in_string = false;
        let mut escape = false;
        while self.current < data.source.len() {
            let c = data.source[self.current];
            if c == '\n' {
                self.line += 1;
            } else if c == '\\' && !escape {
                escape = true;
            } else {
                if c == '\"' && !escape {
                    in_string = !in_string;
                } else if !in_string {
                    if self.matches(multi_end, data) {
                        level -= 1;
                        self.current += multi_end.len() - 1;
                        if level == 0 {
                            return Some(TokenType::Comment(
                                data.source[self.start..self.current - 1]
                                    .iter()
                                    .cloned()
                                    .collect::<String>(),
                            ));
                        }
                    } else if self.matches(multi_start, data) {
                        self.current += multi_start.len() - 1;
                        level += 1;
                    }
                }
                escape = false;
            }
            self.current += 1;
        }
        None
    }
    fn scan_number(&mut self, data: &mut ScannerData) -> Option<TokenType> {
        if is_digit(data.source[self.current]) {
            let source_len = data.source.len();
            if self.current < source_len - 2 {
                if data.source[self.current + 1] == 'x' || data.source[self.current + 1] == 'X' {
                    self.current += 2;
                    return self.scan_hex_number(data);
                } else if data.source[self.current + 1] == 'b'
                    || data.source[self.current + 1] == 'B'
                {
                    self.current += 2;
                    return self.scan_binary_number(data);
                }
            }
            let mut number = 0.0;
            let mut value = String::new();
            while self.current < source_len && is_digit(data.source[self.current]) {
                let c = data.source[self.current];
                value.push(c);
                number = number * 10.0 + Number::from((c as u8) - b'0');
                self.current += 1;
            }
            if self.current < source_len - 1
                && data.source[self.current] == '.'
                && is_digit(data.source[self.current + 1])
            {
                self.current += 1;
                value.push('.');
                let mut div = 1.0;
                while self.current < source_len && is_digit(data.source[self.current]) {
                    let c = data.source[self.current];
                    value.push(c);
                    number = number * 10.0 + Number::from((c as u8) - b'0');
                    self.current += 1;
                    div *= 10.0;
                }
                number /= div;
            }
            return Some(TokenType::NumberLiteral(value, number));
        }
        None
    }
    fn scan_binary_number(&mut self, data: &mut ScannerData) -> Option<TokenType> {
        let mut number = 0.0;
        let mut value = String::new();
        loop {
            let c = data.source[self.current];
            match c {
                '0' | '1' => {
                    number = number * 2.0 + Number::from((c as u8) - b'0');
                    value.push(c);
                }
                _ => break,
            }
            self.current += 1;
            if self.current == data.source.len() {
                break;
            }
        }
        Some(TokenType::NumberLiteral(format!("0b{}", value), number))
    }
    fn scan_hex_number(&mut self, data: &mut ScannerData) -> Option<TokenType> {
        let mut number = 0.0;
        let mut value = String::new();
        loop {
            let c = data.source[self.current];
            match c {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    number = number * 16.0 + Number::from((c as u8) - b'0');
                    value.push(c);
                }
                'a' | 'b' | 'c' | 'd' | 'e' | 'f' => {
                    number = number * 16.0 + Number::from((c as u8) - b'a' + 10);
                    value.push(c);
                }
                'A' | 'B' | 'C' | 'D' | 'E' | 'F' => {
                    number = number * 16.0 + Number::from((c as u8) - b'A' + 10);
                    value.push(c);
                }
                _ => break,
            }
            self.current += 1;
            if self.current == data.source.len() {
                break;
            }
        }
        Some(TokenType::NumberLiteral(format!("0x{}", value), number))
    }
    fn scan_identifier(&mut self, data: &mut ScannerData) -> Option<TokenType> {
        if is_alpha(data.source[self.current]) {
            let mut value = String::new();
            while self.current < data.source.len() && is_alphanum(data.source[self.current]) {
                value.push(data.source[self.current]);
                self.current += 1;
            }
            return Some(TokenType::Identifier(value));
        }
        None
    }
    fn scan_space(&mut self, data: &mut ScannerData) -> Option<TokenType> {
        let start = self.current;
        while self.current < data.source.len() && is_space(data.source[self.current]) {
            self.current += 1;
        }
        if start == self.current {
            return None;
        }
        Some(TokenType::Ignore)
    }
    fn scan_string(&mut self, data: &mut ScannerData) -> Result<Option<TokenType>, ScanError> {
        if data.source[self.current] == '\"' {
            self.current += 1;
            let mut escape = false;
            let mut value = String::new();
            while self.current < data.source.len() {
                let c = data.source[self.current];
                if c == '\\' && !escape {
                    escape = true;
                } else {
                    if c == '\"' && !escape {
                        self.current += 1;
                        return Ok(Some(TokenType::StringLiteral(value)));
                    } else if c == 'n' && escape {
                        value.push('\n');
                    } else if c == 't' && escape {
                        value.push('\t');
                    } else {
                        value.push(c);
                        if c == '\n' {
                            self.line += 1;
                        }
                    }
                    escape = false;
                }
                self.current += 1;
            }
            data.token_len.push(data.source.len() - self.start + 1);
            data.token_start.push(self.start);
            data.token_types.push(TokenType::StringLiteral(value));
            data.token_lines.push(self.line);
            let token_id = data.token_len.len() - 1;
            return Err(ScanError::UnexpectedEof(
                self.line,
                data.token_start[token_id],
            ));
        }
        Ok(None)
    }
    fn scan_newline(&mut self, data: &ScannerData) -> Option<TokenType> {
        if data.source[self.current] == '\n' {
            self.current += 1;
            self.line += 1;
            return Some(TokenType::NewLine);
        }
        None
    }
    fn scan_symbol(&mut self, data: &ScannerData, config: &ScannerConfig) -> Option<TokenType> {
        for s in config.symbols.iter() {
            if self.matches(s, data) {
                self.current += s.len();
                return Some(TokenType::Symbol((*s).to_owned()));
            }
        }
        None
    }
    fn scan_keyword(&mut self, data: &ScannerData, config: &ScannerConfig) -> Option<TokenType> {
        let source_len = data.source.len();
        for s in config.keywords.iter() {
            let keyword_len = s.len();
            if self.matches(s, data)
                && (self.current + keyword_len >= source_len
                    || !is_alphanum(data.source[self.current + keyword_len]))
            {
                self.current += s.len();
                return Some(TokenType::Keyword((*s).to_owned()));
            }
        }
        None
    }
    fn matches(&self, s: &str, data: &ScannerData) -> bool {
        let mut check = true;
        let source_len = data.source.len();
        for (i, c) in s.chars().enumerate() {
            if self.current + i >= source_len || data.source[self.current + i] != c {
                check = false;
                break;
            }
        }
        check
    }
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

fn is_alphanum(c: char) -> bool {
    is_digit(c) || is_alpha(c)
}

fn is_space(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\r'
}
