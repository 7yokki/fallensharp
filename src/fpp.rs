//! Fallen++ ön yüzü: girinti/`do`/`end` destekli basit sözdizimini
//! mevcut Fallen AST'sine indirger. Böylece olgun x86-64 backend yeniden kullanılır.

use crate::ast::*;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
enum Tk {
    Ident(String),
    Int(i64),
    Str(Vec<u8>),
    Char(i64),
    Punct(String),
    Newline,
    Eof,
}

#[derive(Clone, Debug)]
struct Token {
    kind: Tk,
    line: usize,
    col: usize,
}

struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer {
            src: src.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn peek_at(&self, n: usize) -> Option<u8> {
        self.src.get(self.pos + n).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let c = self.peek()?;
        self.pos += 1;
        if c == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let mut out = Vec::new();
        while let Some(c) = self.peek() {
            match c {
                b' ' | b'\t' | b'\r' => {
                    self.bump();
                }
                b'\n' => {
                    let line = self.line;
                    let col = self.col;
                    self.bump();
                    out.push(Token {
                        kind: Tk::Newline,
                        line,
                        col,
                    });
                }
                b'#' => {
                    while let Some(ch) = self.bump() {
                        if ch == b'\n' {
                            let line = self.line - 1;
                            out.push(Token {
                                kind: Tk::Newline,
                                line,
                                col: 1,
                            });
                            break;
                        }
                    }
                }
                b'"' => {
                    let line = self.line;
                    let col = self.col;
                    self.bump();
                    let bytes = self.lex_string()?;
                    out.push(Token {
                        kind: Tk::Str(bytes),
                        line,
                        col,
                    });
                }
                b'\'' => {
                    let line = self.line;
                    let col = self.col;
                    self.bump();
                    let v = self.lex_char()?;
                    out.push(Token {
                        kind: Tk::Char(v),
                        line,
                        col,
                    });
                }
                c if c.is_ascii_alphabetic() || c == b'_' => {
                    let line = self.line;
                    let col = self.col;
                    let start = self.pos;
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_alphanumeric() || ch == b'_' {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    let s = String::from_utf8_lossy(&self.src[start..self.pos]).to_string();
                    out.push(Token {
                        kind: Tk::Ident(s),
                        line,
                        col,
                    });
                }
                c if c.is_ascii_digit() => {
                    let line = self.line;
                    let col = self.col;
                    let v = self.lex_number()?;
                    out.push(Token {
                        kind: Tk::Int(v),
                        line,
                        col,
                    });
                }
                b';' => {
                    let line = self.line;
                    let col = self.col;
                    self.bump();
                    out.push(Token {
                        kind: Tk::Newline,
                        line,
                        col,
                    });
                }
                _ => {
                    let line = self.line;
                    let col = self.col;
                    let p = self.lex_punct()?;
                    out.push(Token {
                        kind: Tk::Punct(p),
                        line,
                        col,
                    });
                }
            }
        }
        out.push(Token {
            kind: Tk::Eof,
            line: self.line,
            col: self.col,
        });
        Ok(out)
    }

    fn lex_number(&mut self) -> Result<i64, String> {
        let start = self.pos;
        if self.peek() == Some(b'0') {
            match self.peek_at(1) {
                Some(b'x') | Some(b'X') => {
                    self.bump();
                    self.bump();
                    let s = self.pos;
                    while let Some(c) = self.peek() {
                        if c.is_ascii_hexdigit() || c == b'_' {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    let text: String = std::str::from_utf8(&self.src[s..self.pos])
                        .unwrap_or("")
                        .replace('_', "");
                    return i64::from_str_radix(&text, 16)
                        .map_err(|_| format!("satır {}: geçersiz onaltılık sayı", self.line));
                }
                Some(b'b') | Some(b'B') => {
                    self.bump();
                    self.bump();
                    let s = self.pos;
                    while let Some(c) = self.peek() {
                        if c == b'0' || c == b'1' || c == b'_' {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    let text: String = std::str::from_utf8(&self.src[s..self.pos])
                        .unwrap_or("")
                        .replace('_', "");
                    return i64::from_str_radix(&text, 2)
                        .map_err(|_| format!("satır {}: geçersiz ikilik sayı", self.line));
                }
                Some(b'o') | Some(b'O') => {
                    self.bump();
                    self.bump();
                    let s = self.pos;
                    while let Some(c) = self.peek() {
                        if (b'0'..=b'7').contains(&c) || c == b'_' {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    let text: String = std::str::from_utf8(&self.src[s..self.pos])
                        .unwrap_or("")
                        .replace('_', "");
                    return i64::from_str_radix(&text, 8)
                        .map_err(|_| format!("satır {}: geçersiz sekizlik sayı", self.line));
                }
                _ => {}
            }
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'_' {
                self.bump();
            } else {
                break;
            }
        }
        let text: String = std::str::from_utf8(&self.src[start..self.pos])
            .unwrap_or("")
            .replace('_', "");
        let trimmed = text.trim_end_matches(|c: char| c.is_ascii_alphabetic());
        trimmed
            .parse::<i64>()
            .map_err(|_| format!("satır {}: geçersiz sayı '{}'", self.line, text))
    }

    fn lex_escape(&mut self) -> Result<Vec<u8>, String> {
        match self.bump() {
            Some(b'n') => Ok(vec![b'\n']),
            Some(b't') => Ok(vec![b'\t']),
            Some(b'r') => Ok(vec![b'\r']),
            Some(b'0') => Ok(vec![0]),
            Some(b'\\') => Ok(vec![b'\\']),
            Some(b'"') => Ok(vec![b'"']),
            Some(b'\'') => Ok(vec![b'\'']),
            Some(b'x') => {
                let mut v = 0u8;
                for _ in 0..2 {
                    let c = self
                        .bump()
                        .ok_or_else(|| "kaçış dizisinde eksik onaltılık".to_string())?;
                    let d = (c as char)
                        .to_digit(16)
                        .ok_or_else(|| "geçersiz onaltılık kaçış".to_string())?;
                    v = v * 16 + d as u8;
                }
                Ok(vec![v])
            }
            Some(other) => Ok(vec![other]),
            None => Err("dosya sonunda bitmemiş kaçış dizisi".into()),
        }
    }

    fn lex_string(&mut self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        loop {
            match self.bump() {
                Some(b'"') => return Ok(out),
                Some(b'\\') => out.extend(self.lex_escape()?),
                Some(b'\n') => return Err(format!("satır {}: string satır sonunda bitmedi", self.line - 1)),
                Some(c) => out.push(c),
                None => return Err("dosya sonunda bitmemiş string".into()),
            }
        }
    }

    fn lex_char(&mut self) -> Result<i64, String> {
        let v = match self.bump() {
            Some(b'\\') => self.lex_escape()?,
            Some(c) => vec![c],
            None => return Err("dosya sonunda bitmemiş karakter".into()),
        };
        if self.bump() != Some(b'\'') {
            return Err(format!("satır {}: karakter literali kapatılmadı", self.line));
        }
        Ok(v.first().copied().unwrap_or(0) as i64)
    }

    fn lex_punct(&mut self) -> Result<String, String> {
        let two: &[&str] = &[
            "=>", "->", "==", "!=", "<=", ">=", "&&", "||", "<<", ">>", "+=", "-=", "*=", "/=",
            "%=", "++", "--",
        ];
        if let (Some(a), Some(b)) = (self.peek(), self.peek_at(1)) {
            let s = [a, b];
            for p in two {
                if s == p.as_bytes() {
                    self.bump();
                    self.bump();
                    return Ok((*p).to_string());
                }
            }
        }
        let c = self.bump().unwrap();
        let p = (c as char).to_string();
        if "+-*/%<>=!&|^~()[]{}:,.?".contains(c as char) {
            Ok(p)
        } else {
            Err(format!("satır {}: beklenmeyen karakter '{}'", self.line, p))
        }
    }
}

enum BlockEnd {
    Term(String),
    Dedent,
    Eof,
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

type PResult<T> = Result<T, String>;

impl Parser {
    fn new(toks: Vec<Token>) -> Self {
        Parser { toks, pos: 0 }
    }

    fn peek(&self) -> &Tk {
        &self.toks[self.pos].kind
    }

    fn peek_at(&self, n: usize) -> &Tk {
        if self.pos + n < self.toks.len() {
            &self.toks[self.pos + n].kind
        } else {
            &Tk::Eof
        }
    }

    fn token(&self) -> &Token {
        &self.toks[self.pos]
    }

    fn line(&self) -> usize {
        self.toks[self.pos].line
    }

    fn col(&self) -> usize {
        self.toks[self.pos].col
    }

    fn advance(&mut self) -> Tk {
        let k = self.toks[self.pos].kind.clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        k
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Tk::Eof)
    }

    fn is_ident(&self, s: &str) -> bool {
        matches!(self.peek(), Tk::Ident(x) if x == s)
    }

    fn is_ident_at(&self, n: usize, s: &str) -> bool {
        matches!(self.peek_at(n), Tk::Ident(x) if x == s)
    }

    fn eat_ident(&mut self, s: &str) -> bool {
        if self.is_ident(s) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self, s: &str) -> PResult<()> {
        if self.eat_ident(s) {
            Ok(())
        } else {
            Err(self.err(format!("'{}' bekleniyordu", s)))
        }
    }

    fn expect_name(&mut self) -> PResult<String> {
        match self.peek().clone() {
            Tk::Ident(s) => {
                self.advance();
                Ok(s)
            }
            other => Err(self.err(format!("tanımlayıcı bekleniyordu, bulunan {:?}", other))),
        }
    }

    fn is_punct(&self, p: &str) -> bool {
        matches!(self.peek(), Tk::Punct(s) if s == p)
    }

    fn is_punct_at(&self, n: usize, p: &str) -> bool {
        matches!(self.peek_at(n), Tk::Punct(s) if s == p)
    }

    fn eat_punct(&mut self, p: &str) -> bool {
        if self.is_punct(p) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_punct(&mut self, p: &str) -> PResult<()> {
        if self.eat_punct(p) {
            Ok(())
        } else {
            Err(self.err(format!("'{}' bekleniyordu", p)))
        }
    }

    fn err(&self, msg: String) -> String {
        format!("Fallen++ satır {}:{}: {}", self.line(), self.col(), msg)
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Tk::Newline) {
            self.advance();
        }
    }

    fn at_term(&self, terms: &[&str]) -> Option<String> {
        if let Tk::Ident(s) = self.peek() {
            if terms.iter().any(|t| t == s) {
                return Some(s.clone());
            }
        }
        None
    }

    fn is_top_keyword(&self) -> bool {
        matches!(self.peek(), Tk::Ident(s) if s == "fn" || s == "class" || s == "import")
    }

    pub fn parse_program(mut self) -> PResult<Program> {
        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.at_eof() {
                break;
            }
            if self.is_ident("import") {
                self.parse_import()?;
            } else if self.is_ident("fn") {
                items.push(Item::Func(self.parse_fn()?));
            } else if self.is_ident("class") {
                items.push(Item::Class(self.parse_class()?));
            } else {
                return Err(self.err(
                    "üst düzeyde yalnızca 'import', 'fn' veya 'class' kullanılabilir".into(),
                ));
            }
        }
        Ok(Program { items })
    }

    fn parse_import(&mut self) -> PResult<()> {
        self.expect_ident("import")?;
        let mut path = self.expect_name()?;
        while self.eat_punct(".") {
            path.push('.');
            path.push_str(&self.expect_name()?);
        }
        if path.is_empty() {
            return Err(self.err("import yolu boş".into()));
        }
        while !matches!(self.peek(), Tk::Newline | Tk::Eof) {
            self.advance();
        }
        Ok(())
    }

    fn parse_fn(&mut self) -> PResult<FuncDecl> {
        let line = self.line();
        self.expect_ident("fn")?;
        let name = self.expect_name()?;
        self.expect_punct("(")?;
        let mut params = Vec::new();
        self.skip_newlines();
        if !self.is_punct(")") {
            loop {
                self.skip_newlines();
                let pline = self.line();
                let pname = self.expect_name()?;
                if self.eat_punct(":") {
                    // x: int
                }
                let ty = if self.next_is_type() {
                    self.parse_type()?
                } else {
                    Type::Int
                };
                let (ty, bound) = match ty {
                    Type::Array(elem, n) => (Type::Ptr(elem), Some(n)),
                    other => (other, None),
                };
                params.push(VarDecl {
                    name: pname,
                    ty,
                    init: None,
                    ctor_args: Vec::new(),
                    offset: None,
                    bound,
                    line: pline,
                });
                self.skip_newlines();
                if !self.eat_punct(",") {
                    break;
                }
            }
        }
        self.expect_punct(")")?;
        let has_ret_annot = self.eat_punct("->");
        let mut ret = if has_ret_annot {
            self.parse_type()?
        } else {
            Type::Void
        };
        self.expect_punct(":")?;
        if !matches!(self.peek(), Tk::Newline | Tk::Eof) {
            return Err(self.err("fonksiyon başlığından sonra satır sonu bekleniyordu".into()));
        }
        let header_col = self.col();
        self.skip_newlines();
        let (mut body, end) = self.parse_block(header_col, &["end"])?;
        if let BlockEnd::Term(t) = end {
            if t == "end" {
                self.advance();
                self.eat_punct(";");
            }
        }
        if name == "main" && !has_ret_annot {
            ret = Type::Int;
            body.push(Stmt::Return(Some(Expr::new(ExprKind::Int(0), line))));
        }
        Ok(FuncDecl {
            name,
            ret,
            params,
            body: Stmt::Block(body),
            owner: None,
            is_ctor: false,
            is_method: false,
            is_private: false,
            is_prototype: false,
            frame_size: 0,
            this_offset: None,
            line,
        })
    }

    fn parse_class(&mut self) -> PResult<ClassDecl> {
        let line = self.line();
        self.expect_ident("class")?;
        let name = self.expect_name()?;
        self.expect_punct(":")?;
        let header_col = self.col();
        self.skip_newlines();
        let (body, _end) = self.parse_block(header_col, &["end"])?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut is_private = true;
        for st in body {
            match st {
                Stmt::Var(v) => fields.push(FieldDecl {
                    name: v.name,
                    ty: v.ty,
                    is_private,
                    line: v.line,
                }),
                Stmt::Expr(e) => {
                    if let ExprKind::Ident(s) = &e.kind {
                        match s.as_str() {
                            "private" => is_private = true,
                            "public" => is_private = false,
                            _ => {
                                return Err(format!(
                                    "Fallen++ satır {}: sınıf gövdesinde geçersiz öğe '{}'",
                                    e.line, s
                                ))
                            }
                        }
                    }
                }
                Stmt::Group(items) => {
                    for it in items {
                        if let Stmt::Var(v) = it {
                            fields.push(FieldDecl {
                                name: v.name,
                                ty: v.ty,
                                is_private,
                                line: v.line,
                            });
                        }
                    }
                }
                _ => {
                    return Err(format!(
                        "Fallen++ satır {}: sınıf gövdesinde yalnızca alanlar ve erişim etiketleri destekleniyor",
                        line
                    ))
                }
            }
        }
        // Metotlar ilk sürümde sınıf gövdesinde değil, ayrı fonksiyon olarak ele alınır.
        let _ = &mut methods;
        Ok(ClassDecl {
            name,
            fields,
            methods,
            line,
        })
    }

    fn parse_block(&mut self, header_col: usize, terms: &[&str]) -> PResult<(Vec<Stmt>, BlockEnd)> {
        self.skip_newlines();
        if let Some(t) = self.at_term(terms) {
            return Ok((Vec::new(), BlockEnd::Term(t)));
        }
        if self.at_eof() {
            return Ok((Vec::new(), BlockEnd::Eof));
        }
        let indented = self.col() > header_col;
        let explicit = !indented;
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if self.at_eof() {
                return Ok((stmts, BlockEnd::Eof));
            }
            if let Some(t) = self.at_term(terms) {
                return Ok((stmts, BlockEnd::Term(t)));
            }
            if !explicit && self.col() <= header_col {
                return Ok((stmts, BlockEnd::Dedent));
            }
            if explicit && !stmts.is_empty() && self.col() <= header_col && self.is_top_keyword() {
                return Ok((stmts, BlockEnd::Dedent));
            }
            let st = self.parse_stmt()?;
            stmts.push(st);
        }
    }

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        self.skip_newlines();
        if self.is_ident("if") {
            return self.parse_if();
        }
        if self.is_ident("loop") {
            return self.parse_loop();
        }
        if self.is_ident("while") {
            return self.parse_while();
        }
        if self.is_ident("return") {
            self.advance();
            if matches!(self.peek(), Tk::Newline | Tk::Eof) {
                return Ok(Stmt::Return(None));
            }
            let e = self.parse_expr()?;
            return Ok(Stmt::Return(Some(e)));
        }
        if self.is_ident("break") {
            let line = self.line();
            self.advance();
            return Ok(Stmt::Break(line));
        }
        if self.is_ident("continue") {
            let line = self.line();
            self.advance();
            return Ok(Stmt::Continue(line));
        }
        if self.is_ident("end") || self.is_ident("do") || self.is_ident("else") {
            return Err(self.err(format!(
                "beklenmeyen blok sonlandırıcı '{}'",
                if let Tk::Ident(s) = self.peek() { s } else { "" }
            )));
        }
        if self.looks_like_binding() {
            return self.parse_binding();
        }
        let e = self.parse_expr()?;
        Ok(Stmt::Expr(e))
    }

    fn looks_like_binding(&self) -> bool {
        let name = match self.peek() {
            Tk::Ident(s) => s,
            _ => return false,
        };
        if name == "if" || name == "loop" || name == "while" || name == "return" {
            return false;
        }
        if self.is_punct_at(1, "=>") || self.is_punct_at(1, ":") {
            return true;
        }
        if let Tk::Ident(t) = self.peek_at(1) {
            if Self::type_word(t) {
                return true;
            }
        }
        false
    }

    fn parse_binding(&mut self) -> PResult<Stmt> {
        let line = self.line();
        let name = self.expect_name()?;
        let mut ty = None;
        if self.eat_punct(":") {
            ty = Some(self.parse_type()?);
        } else if self.next_is_type() {
            ty = Some(self.parse_type()?);
        }
        let init = if self.eat_punct("=>") || self.eat_punct("=") {
            Some(self.parse_expr()?)
        } else {
            None
        };
        let ty = ty.unwrap_or_else(|| infer_type(init.as_ref()));
        Ok(Stmt::Var(VarDecl {
            name,
            ty,
            init,
            ctor_args: Vec::new(),
            offset: None,
            bound: None,
            line,
        }))
    }

    fn parse_if(&mut self) -> PResult<Stmt> {
        self.expect_ident("if")?;
        let cond = self.parse_expr()?;
        self.expect_punct(":")?;
        let header_col = self.col();
        self.skip_newlines();
        let (then_body, end) = self.parse_block(header_col, &["else", "end"])?;
        let then_stmt = Stmt::Block(then_body);
        let mut else_stmt = None;
        match end {
            BlockEnd::Term(t) if t == "else" => {
                self.advance();
                if self.is_ident("if") {
                    else_stmt = Some(Box::new(self.parse_if()?));
                } else {
                    self.expect_punct(":")?;
                    let else_col = self.col();
                    self.skip_newlines();
                    let (else_body, else_end) = self.parse_block(else_col, &["end"])?;
                    if let BlockEnd::Term(t) = else_end {
                        if t == "end" {
                            self.advance();
                        }
                    }
                    else_stmt = Some(Box::new(Stmt::Block(else_body)));
                }
            }
            BlockEnd::Term(t) if t == "end" => {
                self.advance();
            }
            _ => {}
        }
        Ok(Stmt::If(cond, Box::new(then_stmt), else_stmt))
    }

    fn parse_loop(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_ident("loop")?;
        self.expect_punct(":")?;
        let header_col = self.col();
        self.skip_newlines();
        let (body, end) = self.parse_block(header_col, &["do", "end"])?;
        if let BlockEnd::Term(_) = end {
            self.advance();
        }
        let cond = Expr::new(ExprKind::Int(1), line);
        Ok(Stmt::While(cond, Box::new(Stmt::Block(body))))
    }

    fn parse_while(&mut self) -> PResult<Stmt> {
        self.expect_ident("while")?;
        let cond = self.parse_expr()?;
        self.expect_punct(":")?;
        let header_col = self.col();
        self.skip_newlines();
        let (body, end) = self.parse_block(header_col, &["end"])?;
        if let BlockEnd::Term(_) = end {
            self.advance();
        }
        Ok(Stmt::While(cond, Box::new(Stmt::Block(body))))
    }

    fn next_is_type(&self) -> bool {
        match self.peek() {
            Tk::Ident(s) => Self::type_word(s),
            _ => false,
        }
    }

    fn type_word(s: &str) -> bool {
        matches!(
            s,
            "void"
                | "int"
                | "uint"
                | "long"
                | "ulong"
                | "short"
                | "ushort"
                | "char"
                | "uchar"
                | "bool"
                | "string"
                | "float"
                | "double"
                | "byte"
                | "ptr"
        )
    }

    fn parse_type(&mut self) -> PResult<Type> {
        if self.eat_punct("*") {
            let inner = self.parse_type()?;
            return Ok(Type::Ptr(Box::new(inner)));
        }
        if self.eat_punct("[") {
            let inner = self.parse_type()?;
            self.expect_punct(";")?;
            let n = match self.advance() {
                Tk::Int(v) if v > 0 => v as usize,
                _ => return Err(self.err("dizi boyutu pozitif tam sayı olmalı".into())),
            };
            self.expect_punct("]")?;
            return Ok(Type::Array(Box::new(inner), n));
        }
        let name = self.expect_name()?;
        let base = match name.as_str() {
            "void" => Type::Void,
            "int" | "bool" | "float" | "double" => Type::Int,
            "uint" => Type::UInt,
            "long" => Type::Long,
            "ulong" => Type::ULong,
            "short" => Type::Short,
            "ushort" => Type::UShort,
            "char" | "byte" => Type::Char,
            "uchar" => Type::UChar,
            "string" => Type::ptr(Type::Char),
            "ptr" => Type::Ptr(Box::new(self.parse_type()?)),
            other => Type::Class(other.to_string()),
        };
        if self.eat_punct("[") {
            let n = match self.advance() {
                Tk::Int(v) if v > 0 => v as usize,
                _ => return Err(self.err("dizi boyutu pozitif tam sayı olmalı".into())),
            };
            self.expect_punct("]")?;
            return Ok(Type::Array(Box::new(base), n));
        }
        Ok(base)
    }

    fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_assign()
    }

    fn parse_assign(&mut self) -> PResult<Expr> {
        let left = self.parse_or()?;
        if self.eat_punct("=") {
            let line = self.line();
            let right = self.parse_assign()?;
            return Ok(Expr::new(
                ExprKind::Assign(Box::new(left), Box::new(right)),
                line,
            ));
        }
        if self.eat_punct("+=") {
            let line = self.line();
            let right = self.parse_assign()?;
            return Ok(Expr::new(
                ExprKind::CompoundAssign(BinOp::Add, Box::new(left), Box::new(right)),
                line,
            ));
        }
        if self.eat_punct("-=") {
            let line = self.line();
            let right = self.parse_assign()?;
            return Ok(Expr::new(
                ExprKind::CompoundAssign(BinOp::Sub, Box::new(left), Box::new(right)),
                line,
            ));
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        let mut left = self.parse_and()?;
        loop {
            if self.eat_punct("||") || self.eat_ident("or") {
                let line = self.line();
                let right = self.parse_and()?;
                left = Expr::new(ExprKind::Bin(BinOp::LogOr, Box::new(left), Box::new(right)), line);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        let mut left = self.parse_bitor()?;
        loop {
            if self.eat_punct("&&") || self.eat_ident("and") {
                let line = self.line();
                let right = self.parse_bitor()?;
                left = Expr::new(ExprKind::Bin(BinOp::LogAnd, Box::new(left), Box::new(right)), line);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_bitor(&mut self) -> PResult<Expr> {
        let mut left = self.parse_bitxor()?;
        while self.eat_punct("|") {
            let line = self.line();
            let right = self.parse_bitxor()?;
            left = Expr::new(ExprKind::Bin(BinOp::BitOr, Box::new(left), Box::new(right)), line);
        }
        Ok(left)
    }

    fn parse_bitxor(&mut self) -> PResult<Expr> {
        let mut left = self.parse_bitand()?;
        while self.eat_punct("^") {
            let line = self.line();
            let right = self.parse_bitand()?;
            left = Expr::new(ExprKind::Bin(BinOp::BitXor, Box::new(left), Box::new(right)), line);
        }
        Ok(left)
    }

    fn parse_bitand(&mut self) -> PResult<Expr> {
        let mut left = self.parse_equality()?;
        while self.eat_punct("&") {
            let line = self.line();
            let right = self.parse_equality()?;
            left = Expr::new(ExprKind::Bin(BinOp::BitAnd, Box::new(left), Box::new(right)), line);
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> PResult<Expr> {
        let mut left = self.parse_compare()?;
        loop {
            let op = if self.eat_punct("==") || self.eat_ident("equal") || self.eat_ident("is") {
                BinOp::Eq
            } else if self.eat_punct("!=")
                || (self.is_ident("not") && self.is_ident_at(1, "equal"))
            {
                if self.is_ident("not") {
                    self.advance();
                    self.advance();
                }
                BinOp::Ne
            } else {
                break;
            };
            let line = self.line();
            let right = self.parse_compare()?;
            left = Expr::new(ExprKind::Bin(op, Box::new(left), Box::new(right)), line);
        }
        Ok(left)
    }

    fn parse_compare(&mut self) -> PResult<Expr> {
        let mut left = self.parse_shift()?;
        loop {
            let op = if self.eat_punct("<=")
                || (self.is_ident("less") && self.is_ident_at(1, "equal"))
            {
                if self.is_ident("less") {
                    self.advance();
                    self.advance();
                }
                BinOp::Le
            } else if self.eat_punct(">=")
                || (self.is_ident("greater") && self.is_ident_at(1, "equal"))
            {
                if self.is_ident("greater") {
                    self.advance();
                    self.advance();
                }
                BinOp::Ge
            } else if self.eat_punct("<") || self.eat_ident("less") {
                BinOp::Lt
            } else if self.eat_punct(">") || self.eat_ident("greater") {
                BinOp::Gt
            } else {
                break;
            };
            let line = self.line();
            let right = self.parse_shift()?;
            left = Expr::new(ExprKind::Bin(op, Box::new(left), Box::new(right)), line);
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> PResult<Expr> {
        let mut left = self.parse_add()?;
        loop {
            let op = if self.eat_punct("<<") {
                BinOp::Shl
            } else if self.eat_punct(">>") {
                BinOp::Shr
            } else {
                break;
            };
            let line = self.line();
            let right = self.parse_add()?;
            left = Expr::new(ExprKind::Bin(op, Box::new(left), Box::new(right)), line);
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> PResult<Expr> {
        let mut left = self.parse_mul()?;
        loop {
            let op = if self.eat_punct("+") {
                BinOp::Add
            } else if self.eat_punct("-") {
                BinOp::Sub
            } else {
                break;
            };
            let line = self.line();
            let right = self.parse_mul()?;
            left = Expr::new(ExprKind::Bin(op, Box::new(left), Box::new(right)), line);
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> PResult<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let op = if self.eat_punct("*") {
                BinOp::Mul
            } else if self.eat_punct("/") {
                BinOp::Div
            } else if self.eat_punct("%") {
                BinOp::Mod
            } else {
                break;
            };
            let line = self.line();
            let right = self.parse_unary()?;
            left = Expr::new(ExprKind::Bin(op, Box::new(left), Box::new(right)), line);
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        let line = self.line();
        if self.eat_punct("-") {
            let inner = self.parse_unary()?;
            return Ok(Expr::new(ExprKind::Un(UnOp::Neg, Box::new(inner)), line));
        }
        if self.eat_punct("!") || self.eat_ident("not") {
            let inner = self.parse_unary()?;
            return Ok(Expr::new(ExprKind::Un(UnOp::Not, Box::new(inner)), line));
        }
        if self.eat_punct("~") {
            let inner = self.parse_unary()?;
            return Ok(Expr::new(ExprKind::Un(UnOp::BitNot, Box::new(inner)), line));
        }
        if self.eat_punct("&") {
            let inner = self.parse_unary()?;
            return Ok(Expr::new(ExprKind::Un(UnOp::AddrOf, Box::new(inner)), line));
        }
        if self.eat_punct("*") {
            let inner = self.parse_unary()?;
            return Ok(Expr::new(ExprKind::Un(UnOp::Deref, Box::new(inner)), line));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        let mut e = self.parse_primary()?;
        loop {
            if self.eat_punct(".") {
                let line = self.line();
                let field = self.expect_name()?;
                e = Expr::new(
                    ExprKind::Member {
                        base: Box::new(e),
                        field,
                        arrow: false,
                    },
                    line,
                );
            } else if self.eat_punct("(") {
                let line = self.line();
                let (args, kwargs) = self.parse_args()?;
                e = self.lower_call(e, args, kwargs, line);
            } else if self.eat_punct("[") {
                let line = self.line();
                let idx = self.parse_expr()?;
                self.expect_punct("]")?;
                e = Expr::new(ExprKind::Index(Box::new(e), Box::new(idx)), line);
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn parse_args(&mut self) -> PResult<(Vec<Expr>, Vec<(String, Expr)>)> {
        let mut args = Vec::new();
        let mut kwargs = Vec::new();
        self.skip_newlines();
        if self.eat_punct(")") {
            return Ok((args, kwargs));
        }
        loop {
            self.skip_newlines();
            if let Tk::Ident(name) = self.peek().clone() {
                if self.is_punct_at(1, "=") {
                    self.advance();
                    self.advance();
                    let e = self.parse_expr()?;
                    kwargs.push((name, e));
                    self.skip_newlines();
                    if self.eat_punct(",") {
                        continue;
                    }
                    self.expect_punct(")")?;
                    break;
                }
            }
            let e = self.parse_expr()?;
            args.push(e);
            self.skip_newlines();
            if self.eat_punct(",") {
                continue;
            }
            self.expect_punct(")")?;
            break;
        }
        Ok((args, kwargs))
    }

    fn lower_call(
        &self,
        callee: Expr,
        args: Vec<Expr>,
        kwargs: Vec<(String, Expr)>,
        line: usize,
    ) -> Expr {
        let mut name: Option<String> = None;
        let mut is_member = false;
        match &callee.kind {
            ExprKind::Ident(n) => name = Some(n.clone()),
            ExprKind::Member { base, field, .. } => {
                if let ExprKind::Ident(m) = &base.kind {
                    name = Some(format!("{}.{}", m, field));
                    is_member = true;
                }
            }
            _ => {}
        }
        let _ = is_member;
        let label = match name.as_deref() {
            Some("stdout") => "__fallen_stdout".to_string(),
            Some("stderr") => "__fallen_stderr".to_string(),
            Some("stdin") => "__fallen_readline".to_string(),
            Some("alloc") => "malloc".to_string(),
            Some("free") => "free".to_string(),
            Some("system.exec") => "__fallen_exec".to_string(),
            Some(n) if n.starts_with("fncall.") => n["fncall.".len()..].to_string(),
            Some(n) if n.contains('.') => n.replace('.', "_"),
            Some(n) => n.to_string(),
            None => "__invalid_call".to_string(),
        };

        if label == "__fallen_exec" {
            let mut cmd = None;
            let mut capture = None;
            for a in args {
                if cmd.is_none() {
                    cmd = Some(a);
                } else if capture.is_none() {
                    capture = Some(a);
                }
            }
            for (k, v) in kwargs {
                if k == "output" {
                    capture = Some(v);
                }
            }
            let cmd = cmd.unwrap_or_else(|| Expr::new(ExprKind::Str(Vec::new()), line));
            let capture = capture.unwrap_or_else(|| Expr::new(ExprKind::Int(0), line));
            let callee = Expr::new(ExprKind::Ident("__fallen_exec".into()), line);
            return Expr::new(ExprKind::Call(Box::new(callee), vec![cmd, capture]), line);
        }

        let mut flat = args;
        for (_, v) in kwargs {
            flat.push(v);
        }
        let callee = Expr::new(ExprKind::Ident(label), line);
        Expr::new(ExprKind::Call(Box::new(callee), flat), line)
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let line = self.line();
        match self.peek().clone() {
            Tk::Int(v) => {
                self.advance();
                Ok(Expr::new(ExprKind::Int(v), line))
            }
            Tk::Str(b) => {
                self.advance();
                Ok(Expr::new(ExprKind::Str(b), line))
            }
            Tk::Char(v) => {
                self.advance();
                Ok(Expr::new(ExprKind::Int(v), line))
            }
            Tk::Ident(s) => {
                self.advance();
                match s.as_str() {
                    "true" | "True" => Ok(Expr::new(ExprKind::Int(1), line)),
                    "false" | "False" | "null" | "nil" | "None" => {
                        Ok(Expr::new(ExprKind::Int(0), line))
                    }
                    _ => Ok(Expr::new(ExprKind::Ident(s), line)),
                }
            }
            Tk::Punct(p) if p == "(" => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect_punct(")")?;
                Ok(e)
            }
            other => Err(self.err(format!("ifade bekleniyordu, bulunan {:?}", other))),
        }
    }
}

fn infer_type(init: Option<&Expr>) -> Type {
    match init.map(|e| &e.kind) {
        Some(ExprKind::Str(_)) => Type::ptr(Type::Char),
        Some(ExprKind::Call(callee, _)) => match &callee.kind {
            ExprKind::Ident(n)
                if n == "__fallen_readline" || n == "__fallen_exec" =>
            {
                Type::ptr(Type::Char)
            }
            ExprKind::Ident(n) if n == "__fallen_stdout" || n == "__fallen_stderr" => Type::Void,
            _ => Type::Int,
        },
        Some(ExprKind::Ident(_)) => Type::Int,
        _ => Type::Int,
    }
}

pub fn parse(src: &str) -> Result<Program, String> {
    let toks = Lexer::new(src).tokenize()?;
    Parser::new(toks).parse_program()
}
