use std::collections::HashMap;
use std::path::PathBuf;
use erl_tokenize::{Token, Tokenizer, Position, TokenValue, PositionRange};
use erl_tokenize::tokens::VariableToken;
use erl_tokenize::values::Symbol;

use {Result, Directive, ErrorKind};
use directive::{self, MacroDef, MacroName, Undef};
use token_reader::TokenReader;

#[derive(Debug)]
pub struct Preprocessor<'a> {
    reader: TokenReader<'a>,
    can_directive_start: bool,
    macros: HashMap<MacroName, usize>,
    directives: Vec<Directive>,
    code_paths: Vec<PathBuf>,
}
impl<'a> Preprocessor<'a> {
    pub fn new(tokens: Tokenizer<'a>) -> Self {
        Preprocessor {
            reader: TokenReader::new(tokens),
            can_directive_start: true,
            macros: HashMap::new(),
            directives: Vec::new(),
            code_paths: Vec::new(),
        }
    }
    fn next_token(&mut self) -> Result<Option<Token>> {
        if self.can_directive_start {
            self.reader.start_transaction();
            if let Some(d) = track_try!(self.try_read_directive()) {
                self.directives.push(d);
            } else {
                self.reader.abort_transaction();
            }
        }

        if let Some(token) = track_try!(self.reader.read()) {
            match token {
                Token::Whitespace(_) |
                Token::Comment(_) => {}
                Token::Symbol(ref s) => {
                    self.can_directive_start = s.value() == Symbol::Dot;
                }
                _ => self.can_directive_start = false,
            }
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }
    fn try_read_directive(&mut self) -> Result<Option<Directive>> {
        if track_try!(self.reader.read_symbol_if(Symbol::Hyphen)).is_none() {
            return Ok(None);
        }
        track_try!(self.reader.skip_whitespace_or_comment());

        if let Some(atom) = track_try!(self.reader.read_atom()) {
            match atom.value() {
                "include" => unimplemented!(),
                "include_lib" => unimplemented!(),
                "define" => {
                    let d = track_try!(self.read_define_directive());
                    self.macros.insert(d.name.clone(), self.directives.len());
                    return Ok(Some(Directive::Define(d)));
                }
                "undef" => {
                    let d = track_try!(self.read_undef_directive());
                    self.macros.remove(&d.name);
                    return Ok(Some(Directive::Undef(d)));
                }
                "ifdef" => unimplemented!(),
                "ifndef" => unimplemented!(),
                "else" => unimplemented!(),
                "endif" => unimplemented!(),
                "error" => {
                    let d = track_try!(self.read_error_directive());
                    return Ok(Some(Directive::Error(d)));
                }
                "warning" => {
                    let d = track_try!(self.read_warning_directive());
                    return Ok(Some(Directive::Warning(d)));
                }
                _ => {}
            }
        }

        Ok(None)
    }
    fn read_error_directive(&mut self) -> Result<directive::Error> {
        // '('
        track_try!(self.reader.skip_whitespace_or_comment());
        track_try!(self.reader.read_expected_symbol_or_error(Symbol::OpenParen));

        let message_start = self.reader.position();
        let message_end = track_try!(self.skip_remaining_directive_tokens());

        Ok(directive::Error {
               message_start,
               message_end,
               tokens: self.reader.commit_transaction(),
           })
    }
    fn read_warning_directive(&mut self) -> Result<directive::Warning> {
        // '('
        track_try!(self.reader.skip_whitespace_or_comment());
        track_try!(self.reader.read_expected_symbol_or_error(Symbol::OpenParen));

        let message_start = self.reader.position();
        let message_end = track_try!(self.skip_remaining_directive_tokens());

        Ok(directive::Warning {
               message_start,
               message_end,
               tokens: self.reader.commit_transaction(),
           })
    }
    fn read_parenthesized_macro_name(&mut self) -> Result<MacroName> {
        // '('
        track_try!(self.reader.skip_whitespace_or_comment());
        track_try!(self.reader.read_expected_symbol_or_error(Symbol::OpenParen));

        // macro name
        track_try!(self.reader.skip_whitespace_or_comment());
        let name = track_try!(self.read_macro_name());

        // ')'
        track_try!(self.reader.skip_whitespace_or_comment());
        track_try!(self.reader
                       .read_expected_symbol_or_error(Symbol::CloseParen));

        // '.'
        track_try!(self.reader.skip_whitespace_or_comment());
        track_try!(self.reader.read_expected_symbol_or_error(Symbol::Dot));

        Ok(name)
    }
    fn read_undef_directive(&mut self) -> Result<Undef> {
        let name = track_try!(self.read_parenthesized_macro_name());
        Ok(Undef {
               name,
               tokens: self.reader.commit_transaction(),
           })
    }
    fn read_define_directive(&mut self) -> Result<MacroDef> {
        // '('
        track_try!(self.reader.skip_whitespace_or_comment());
        track_try!(self.reader.read_expected_symbol_or_error(Symbol::OpenParen));

        // macro name
        track_try!(self.reader.skip_whitespace_or_comment());
        let name = track_try!(self.read_macro_name());

        // macro variables
        track_try!(self.reader.skip_whitespace_or_comment());
        let vars = match track_try!(self.reader.read_symbol_or_error()).value() {
            Symbol::Comma => None,
            Symbol::OpenParen => Some(track_try!(self.read_macro_vars())),
            s => {
                track_panic!(ErrorKind::InvalidInput,
                             "Unexpected symbol: actual={:?}, expected=Comma|OpenParent",
                             s)
            }
        };
        let replacement_start = self.reader.position();

        // macro replacement
        let replacement_end = track_try!(self.read_macro_replacement());

        Ok(MacroDef {
               name,
               vars,
               replacement_start,
               replacement_end,
               tokens: self.reader.commit_transaction(),
           })
    }
    fn read_macro_name(&mut self) -> Result<MacroName> {
        if let Some(atom) = track_try!(self.reader.read_atom()) {
            Ok(MacroName::Atom(atom))
        } else if let Some(var) = track_try!(self.reader.read_variable()) {
            Ok(MacroName::Variable(var))
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "Invalid macro name: {:?}",
                         self.reader.read());
        }
    }
    fn read_macro_vars(&mut self) -> Result<Vec<VariableToken>> {
        let mut vars = Vec::new();
        loop {
            track_try!(self.reader.skip_whitespace_or_comment());
            let var = track_try!(self.reader.read_variable_or_error());
            vars.push(var);

            track_try!(self.reader.skip_whitespace_or_comment());
            match track_try!(self.reader.read_symbol_or_error()).value() {
                Symbol::Comma => {}
                Symbol::CloseParen => break,
                s => {
                    track_panic!(ErrorKind::InvalidInput,
                                 "Unexpected symbol: actual={:?}, expected=Comma|CloneParent",
                                 s)
                }
            }
        }
        Ok(vars)
    }
    fn skip_remaining_directive_tokens(&mut self) -> Result<Position> {
        loop {
            let token = track_try!(self.reader.read_or_error());
            if token.value() == TokenValue::Symbol(Symbol::CloseParen) {
                let end = token.start_position().clone();
                track_try!(self.reader.skip_whitespace_or_comment());
                if track_try!(self.reader.read_symbol_if(Symbol::Dot)).is_some() {
                    return Ok(end);
                }
            }
        }
    }
    fn read_macro_replacement(&mut self) -> Result<Position> {
        track!(self.skip_remaining_directive_tokens())
    }
}
impl<'a> Iterator for Preprocessor<'a> {
    type Item = Result<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Err(e) => Some(Err(e)),
            Ok(None) => None,
            Ok(Some(token)) => Some(Ok(token)),
        }
    }
}
