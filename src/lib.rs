//! This crate allows you to package several files in your executable.
//! 
//! This is similar to `include_bin!` but easier to use when you have
//! a lot of files.
//! 
//! Usage:
//! 
//! ```
//! #![feature(phase)]
//! 
//! #[phase(plugin)]
//! extern crate resources_package;
//! 
//! static package: &'static [(&'static [u8], &'static [u8])] = resources_package!([
//!     "path/to/resources/*.png",
//!     "path/to/resources/*.mp3"
//! ]);
//! # fn main() {}
//! ```
//! 
//! The type of the static variable is a slice of (`filename`, `content`). `filename` is
//!  the result of calling `Path::as_vec()`. To turn it back into a path, call
//!  `Path::new(filename)`.
//!
//! **Important**: because of technical reasons, the crate will produce POSIX path if you compile
//!  on POSIX, and Windows path if you compile on Windows. Take care if you send them over the
//!  network.
//!

#![feature(plugin_registrar)]

extern crate glob;
extern crate rustc;
extern crate syntax;

use std::gc::Gc;
use std::io::File;
use std::rc::Rc;
use syntax::ast::{mod, TokenTree};
use syntax::ext::build::AstBuilder;
use syntax::ext::base::{mod, DummyResult, ExtCtxt, MacResult};
use syntax::parse;
use syntax::codemap::Span;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut rustc::plugin::Registry) {
    reg.register_macro("resources_package", macro_handler);
}

fn macro_handler(ecx: &mut ExtCtxt, span: Span, token_tree: &[TokenTree])
    -> Box<MacResult+'static>
{
    // parsing parameters
    let parameters = match base::get_exprs_from_tts(ecx, span, token_tree) {
        Some(v) => v,
        None => return DummyResult::any(span)
    };

    if parameters.len() != 1 {
        ecx.span_err(span, format!("expected 1 argument but got {} (did you forget []?)",
            parameters.len()).as_slice());
        return DummyResult::any(span);
    }

    let parameters: Vec<Path> = {
        use syntax::ast::{ExprLit, ExprVec};

        match parameters[0].node {
            ExprVec(ref list) => {
                // turning each element into a string
                let mut result = Vec::new();
                for element in list.iter() {
                    match base::expr_to_string(ecx, element.clone(), "expected string literal") {
                        Some((s, _)) => result.push(Path::new(s.get().to_string())),
                        None => return DummyResult::any(span)
                    }
                }
                result
            },
            ExprLit(_) => {
                vec![match base::expr_to_string(ecx, parameters[0], "expected string literal") {
                    Some((s, _)) => Path::new(s.get().to_string()),
                    None => return DummyResult::any(span)
                }]
            }
            _ => {
                ecx.span_err(span, format!("wrong format for parameter").as_slice());
                return DummyResult::any(span);
            }
        }
    };

    // the path to the file currently being compiled
    let base_path = {
        let mut base_path = Path::new(ecx.codemap().span_to_filename(span));
        base_path.pop();
        base_path
    };

    // loading the data
    let data: Vec<Gc<ast::Expr>> = {
        let mut data = Vec::new();

        for element in parameters.iter() {
            // turning relative into absolute path
            let pattern = if element.is_absolute() {
                element.clone()
            } else {
                base_path.join(element)
            };

            for path in glob::glob(pattern.as_str().unwrap()) {
                if !path.is_file() {
                    continue;
                }

                let content = match File::open(&path).read_to_end() {
                    Ok(s) => s,
                    Err(e) => {
                        ecx.span_err(span, format!("unable to open {}: {}", path.display(), e)
                            .as_slice());
                        return DummyResult::any(span);
                    }
                };

                let content = content.move_iter().map(|b| ecx.expr_u8(span.clone(), b)).collect();
                let content = ecx.expr_vec_slice(span.clone(), content);

                // adding dependency to the file by creating a parser and dropping it instantly
                ecx.codemap().new_filemap(path.as_str().unwrap().to_string(), "".to_string());

                // getting the path relative from the base_path
                let path = path.path_relative_from(&base_path).unwrap();

                data.push(ecx.expr_tuple(span.clone(), vec![
                    ecx.expr_lit(span.clone(), ast::LitBinary(Rc::new(path.into_vec()))),
                    content
                ]));
            }
        }

        data
    };

    // including data
    base::MacExpr::new(ecx.expr_vec_slice(span.clone(), data))
}
