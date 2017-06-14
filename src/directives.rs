use std::collections::HashMap;
use std::fmt;
use std::path::{PathBuf, Component};
use glob::glob;
use erl_tokenize::{Position, PositionRange, LexicalToken};
use erl_tokenize::tokens::{SymbolToken, AtomToken, StringToken};
use erl_tokenize::values::Symbol;

use Result;
use token_reader::{TokenReader, ReadFrom};
use types::{MacroName, MacroVariables};
use util;

#[derive(Debug, Clone)]
pub struct Include {
    pub _hyphen: SymbolToken,
    pub _include: AtomToken,
    pub _open_paren: SymbolToken,
    pub path: StringToken,
    pub _close_paren: SymbolToken,
    pub _dot: SymbolToken,
}
impl Include {
    pub fn include(&self) -> Result<(PathBuf, String)> {
        let path = track!(util::substitute_path_variables(self.path.value()))?;
        let text = track!(util::read_file(&path))?;
        Ok((path, text))
    }
}
impl PositionRange for Include {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for Include {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-include({}).", self.path.text())
    }
}
impl ReadFrom for Include {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        Ok(Include {
               _hyphen: track!(reader.read_expected(&Symbol::Hyphen))?,
               _include: track!(reader.read_expected("include"))?,
               _open_paren: track!(reader.read_expected(&Symbol::OpenParen))?,
               path: track!(reader.read())?,
               _close_paren: track!(reader.read_expected(&Symbol::CloseParen))?,
               _dot: track!(reader.read_expected(&Symbol::Dot))?,
           })
    }
}

#[derive(Debug, Clone)]
pub struct IncludeLib {
    pub _hyphen: SymbolToken,
    pub _include_lib: AtomToken,
    pub _open_paren: SymbolToken,
    pub path: StringToken,
    pub _close_paren: SymbolToken,
    pub _dot: SymbolToken,
}
impl IncludeLib {
    pub fn include_lib(&self, code_paths: &[PathBuf]) -> Result<(PathBuf, String)> {
        let mut path = track!(util::substitute_path_variables(self.path.value()))?;

        let temp_path = path.clone();
        let mut components = temp_path.components();
        if let Some(Component::Normal(app_name)) = components.next() {
            let app_name = track!(app_name.to_str().ok_or(::Error::invalid_input()))?;
            let pattern = format!("{}-*", app_name);
            'root: for root in code_paths.iter() {
                let pattern = root.join(&pattern);
                let pattern = track!(pattern.to_str().ok_or(::Error::invalid_input()))?;
                for entry in track!(glob(pattern).map_err(::Error::from))? {
                    path = track!(entry.map_err(::Error::from))?;
                    for c in components {
                        path.push(c.as_os_str());
                    }
                    break 'root;
                }
            }
        }

        let text = track!(util::read_file(&path))?;
        Ok((path, text))
    }
}
impl PositionRange for IncludeLib {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for IncludeLib {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-include_lib({}).", self.path.text())
    }
}
impl ReadFrom for IncludeLib {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        Ok(IncludeLib {
               _hyphen: track!(reader.read_expected(&Symbol::Hyphen))?,
               _include_lib: track!(reader.read_expected("include_lib"))?,
               _open_paren: track!(reader.read_expected(&Symbol::OpenParen))?,
               path: track!(reader.read())?,
               _close_paren: track!(reader.read_expected(&Symbol::CloseParen))?,
               _dot: track!(reader.read_expected(&Symbol::Dot))?,
           })
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub _hyphen: SymbolToken,
    pub _error: AtomToken,
    pub _open_paren: SymbolToken,
    pub message: StringToken,
    pub _close_paren: SymbolToken,
    pub _dot: SymbolToken,
}
impl PositionRange for Error {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-error({}).", self.message.text())
    }
}
impl ReadFrom for Error {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        Ok(Error {
               _hyphen: track!(reader.read_expected(&Symbol::Hyphen))?,
               _error: track!(reader.read_expected("error"))?,
               _open_paren: track!(reader.read_expected(&Symbol::OpenParen))?,
               message: track!(reader.read())?,
               _close_paren: track!(reader.read_expected(&Symbol::CloseParen))?,
               _dot: track!(reader.read_expected(&Symbol::Dot))?,
           })
    }
}

#[derive(Debug, Clone)]
pub struct Warning {
    pub _hyphen: SymbolToken,
    pub _warning: AtomToken,
    pub _open_paren: SymbolToken,
    pub message: StringToken,
    pub _close_paren: SymbolToken,
    pub _dot: SymbolToken,
}
impl PositionRange for Warning {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-warning({}).", self.message.text())
    }
}
impl ReadFrom for Warning {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        Ok(Warning {
               _hyphen: track!(reader.read_expected(&Symbol::Hyphen))?,
               _warning: track!(reader.read_expected("warning"))?,
               _open_paren: track!(reader.read_expected(&Symbol::OpenParen))?,
               message: track!(reader.read())?,
               _close_paren: track!(reader.read_expected(&Symbol::CloseParen))?,
               _dot: track!(reader.read_expected(&Symbol::Dot))?,
           })
    }
}

#[derive(Debug, Clone)]
pub struct Endif {
    pub _hyphen: SymbolToken,
    pub _endif: AtomToken,
    pub _dot: SymbolToken,
}
impl PositionRange for Endif {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for Endif {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-endif.")
    }
}
impl ReadFrom for Endif {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        Ok(Endif {
               _hyphen: track!(reader.read_expected(&Symbol::Hyphen))?,
               _endif: track!(reader.read_expected("endif"))?,
               _dot: track!(reader.read_expected(&Symbol::Dot))?,
           })
    }
}

#[derive(Debug, Clone)]
pub struct Else {
    pub _hyphen: SymbolToken,
    pub _else: AtomToken,
    pub _dot: SymbolToken,
}
impl PositionRange for Else {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for Else {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-else.")
    }
}
impl ReadFrom for Else {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        Ok(Else {
               _hyphen: track!(reader.read_expected(&Symbol::Hyphen))?,
               _else: track!(reader.read_expected("else"))?,
               _dot: track!(reader.read_expected(&Symbol::Dot))?,
           })
    }
}

#[derive(Debug, Clone)]
pub struct Undef {
    pub _hyphen: SymbolToken,
    pub _undef: AtomToken,
    pub _open_paren: SymbolToken,
    pub name: MacroName,
    pub _close_paren: SymbolToken,
    pub _dot: SymbolToken,
}
impl PositionRange for Undef {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for Undef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-undef({}).", self.name.text())
    }
}
impl ReadFrom for Undef {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        Ok(Undef {
               _hyphen: track!(reader.read_expected(&Symbol::Hyphen))?,
               _undef: track!(reader.read_expected("undef"))?,
               _open_paren: track!(reader.read_expected(&Symbol::OpenParen))?,
               name: track!(reader.read())?,
               _close_paren: track!(reader.read_expected(&Symbol::CloseParen))?,
               _dot: track!(reader.read_expected(&Symbol::Dot))?,
           })
    }
}

#[derive(Debug, Clone)]
pub struct Ifdef {
    pub _hyphen: SymbolToken,
    pub _ifdef: AtomToken,
    pub _open_paren: SymbolToken,
    pub name: MacroName,
    pub _close_paren: SymbolToken,
    pub _dot: SymbolToken,
}
impl PositionRange for Ifdef {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for Ifdef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-ifdef({}).", self.name.text())
    }
}
impl ReadFrom for Ifdef {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        Ok(Ifdef {
               _hyphen: track!(reader.read_expected(&Symbol::Hyphen))?,
               _ifdef: track!(reader.read_expected("ifdef"))?,
               _open_paren: track!(reader.read_expected(&Symbol::OpenParen))?,
               name: track!(reader.read())?,
               _close_paren: track!(reader.read_expected(&Symbol::CloseParen))?,
               _dot: track!(reader.read_expected(&Symbol::Dot))?,
           })
    }
}

#[derive(Debug, Clone)]
pub struct Ifndef {
    pub _hyphen: SymbolToken,
    pub _ifndef: AtomToken,
    pub _open_paren: SymbolToken,
    pub name: MacroName,
    pub _close_paren: SymbolToken,
    pub _dot: SymbolToken,
}
impl PositionRange for Ifndef {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for Ifndef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-ifndef({}).", self.name.text())
    }
}
impl ReadFrom for Ifndef {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        Ok(Ifndef {
               _hyphen: track!(reader.read_expected(&Symbol::Hyphen))?,
               _ifndef: track!(reader.read_expected("ifndef"))?,
               _open_paren: track!(reader.read_expected(&Symbol::OpenParen))?,
               name: track!(reader.read())?,
               _close_paren: track!(reader.read_expected(&Symbol::CloseParen))?,
               _dot: track!(reader.read_expected(&Symbol::Dot))?,
           })
    }
}

#[derive(Debug, Clone)]
pub struct Define {
    pub _hyphen: SymbolToken,
    pub _define: AtomToken,
    pub _open_paren: SymbolToken,
    pub name: MacroName,
    pub variables: Option<MacroVariables>,
    pub _comma: SymbolToken,
    pub replacement: Vec<LexicalToken>,
    pub _close_paren: SymbolToken,
    pub _dot: SymbolToken,
}
impl Define {
    // TODO:
    pub fn expand(&self, args: Vec<&[LexicalToken]>) -> Result<Vec<LexicalToken>> {
        assert!(self.variables.is_some());
        let vars = self.variables.as_ref().unwrap();
        let binds: HashMap<_, _> = vars.iter().map(|v| v.value()).zip(args.iter()).collect();

        let mut tokens = Vec::new();
        let mut template = self.replacement.iter();
        while let Some(t) = template.next() {
            use erl_tokenize::values::Symbol;

            if let Some(val) = binds.get(t.text()) {
                tokens.extend(val.iter().cloned());
            } else if t.as_symbol_token().map(|t| t.value()) == Some(Symbol::DoubleQuestion) {
                let var = track!(template.next().ok_or(::Error::invalid_input()))?;
                let val = track!(binds.get(var.text()).ok_or(::Error::invalid_input()))?;
                let text = val.iter().map(|t| t.text()).collect::<String>();
                tokens.push(StringToken::from_value(&text, val.first().unwrap().start_position())
                                .into());
            } else {
                tokens.push(t.clone());
            }
        }
        Ok(tokens)
    }
}
impl PositionRange for Define {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
impl fmt::Display for Define {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "-define({}{}, {}).",
               self.name,
               self.variables
                   .as_ref()
                   .map_or("".to_string(), |v| v.to_string()),
               self.replacement
                   .iter()
                   .map(|t| t.text())
                   .collect::<String>())
    }
}
impl ReadFrom for Define {
    fn read_from<T, E>(reader: &mut TokenReader<T, E>) -> Result<Self>
        where T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
              E: Into<::Error>
    {
        let _hyphen = track!(reader.read_expected(&Symbol::Hyphen))?;
        let _define = track!(reader.read_expected("define"))?;
        let _open_paren = track!(reader.read_expected(&Symbol::OpenParen))?;
        let name = track!(reader.read())?;
        let variables = if let Some(token) =
            track!(reader.try_read_expected::<SymbolToken>(&Symbol::OpenParen))? {
            reader.unread_token(token.into());
            Some(track!(reader.read())?)
        } else {
            None
        };
        let _comma = track!(reader.read_expected(&Symbol::Comma))?;

        let mut replacement = Vec::new();
        loop {
            if let Some(_close_paren) = track!(reader.try_read_expected(&Symbol::CloseParen))? {
                if let Some(_dot) = track!(reader.try_read_expected(&Symbol::Dot))? {
                    return Ok(Define {
                                  _hyphen,
                                  _define,
                                  _open_paren,
                                  name,
                                  variables,
                                  _comma,
                                  replacement,
                                  _close_paren,
                                  _dot,
                              });
                }
                replacement.push(_close_paren.into());
            } else {
                let token = track!(reader.read_token())?;
                replacement.push(token);
            }
        }
    }
}
