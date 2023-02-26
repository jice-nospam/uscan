mod scanner;

pub use scanner::*;

#[cfg(test)]
mod tests {
    use crate::{ScannerConfig, ScannerData, Scanner, TokenType, ScanError};
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

    #[test]
    fn it_works() {
        let source_code=r#"
            function test(p1,p2)
                return p1+p2
            end
        "#;

        let mut scanner_data = ScannerData::default();
        Scanner::default().run(source_code, &LUA_CONFIG, &mut scanner_data).unwrap();
        assert_eq!(scanner_data.token_types,&[
            TokenType::Keyword("function".to_string()),
            TokenType::Identifier("test".to_string()),
            TokenType::Symbol("(".to_string()),
            TokenType::Identifier("p1".to_string()),
            TokenType::Symbol(",".to_string()),
            TokenType::Identifier("p2".to_string()),
            TokenType::Symbol(")".to_string()),
            TokenType::Keyword("return".to_string()),
            TokenType::Identifier("p1".to_string()),
            TokenType::Symbol("+".to_string()),
            TokenType::Identifier("p2".to_string()),
            TokenType::Keyword("end".to_string()),
        ]);
        assert_eq!(scanner_data.token_len,&[
            8,4,1,2,1,2,1,6,2,1,2,3
        ]);

    }

    #[test]
    fn unicode_works() {
        let source_code=r#"local s="à" -- comment"#;

        let mut scanner_data = ScannerData::default();
        Scanner::default().run(source_code, &LUA_CONFIG, &mut scanner_data).unwrap();
        assert_eq!(scanner_data.token_types,&[
            TokenType::Keyword("local".to_string()),
            TokenType::Identifier("s".to_string()),
            TokenType::Symbol("=".to_string()),
            TokenType::StringLiteral("à".to_string()),
            TokenType::Comment("-- comment".to_string()),
        ]);
        assert_eq!(scanner_data.token_len,&[
            5,1,1,3,10
        ]);
        assert_eq!(scanner_data.token_start,&[
            0,6,7,8,12
        ]);
        let mut st=String::new();
        for i in 0..5 {
            let s=scanner_data.token_start[i];
            let e = s + scanner_data.token_len[i];
            let text: String = source_code.chars().skip(s).take(e-s).collect();
            st.push_str(&text);
        }
        assert_eq!(&st, "locals=\"à\"-- comment");

    }

    #[test]
    fn while_typing() {
        let source_code=r#"local s="à"#;

        let mut scanner_data = ScannerData::default();
        let res = Scanner::default().run(source_code, &LUA_CONFIG, &mut scanner_data);
        assert_eq!(res,Err(ScanError::UnexpectedEof(1,8)));
        assert_eq!(scanner_data.token_types,&[
            TokenType::Keyword("local".to_string()),
            TokenType::Identifier("s".to_string()),
            TokenType::Symbol("=".to_string()),
            TokenType::StringLiteral("à".to_string()),
        ]);
        assert_eq!(scanner_data.token_len,&[
            5,1,1,3
        ]);
        assert_eq!(scanner_data.token_start,&[
            0,6,7,8
        ]);
        let mut st=String::new();
        for i in 0..4 {
            let s=scanner_data.token_start[i];
            let e = s + scanner_data.token_len[i];
            st.push_str(&source_code[s..e]);
        }
        assert_eq!(&st, "locals=\"à");

    }

    #[test]
    fn multi_comments() {
        let source_code=r#"local s="" --[[comment]]"#;

        let mut scanner_data = ScannerData::default();
        Scanner::default().run(source_code, &LUA_CONFIG, &mut scanner_data).unwrap();
        assert_eq!(scanner_data.token_types,&[
            TokenType::Keyword("local".to_string()),
            TokenType::Identifier("s".to_string()),
            TokenType::Symbol("=".to_string()),
            TokenType::StringLiteral("".to_string()),
            TokenType::Comment("--[[comment]]".to_string()),
        ]);
        assert_eq!(scanner_data.token_len,&[
            5,1,1,2,13
        ]);
        assert_eq!(scanner_data.token_start,&[
            0,6,7,8,11
        ]);
        let mut st=String::new();
        for i in 0..5 {
            let s=scanner_data.token_start[i];
            let e = s + scanner_data.token_len[i];
            st.push_str(&source_code[s..e]);
        }
        assert_eq!(&st, "locals=\"\"--[[comment]]");

    }

}