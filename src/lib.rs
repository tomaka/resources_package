//! This crate allows you to package several files in your executable.
//! 
//! This is similar to `include_bin!` but easier to use when you have
//! a lot of files.
//! 
//! Usage:
//! 
//! ```ignore
//! #![feature(phase)]
//! 
//! #[phase(plugin)]
//! extern crate resources_package;
//! 
//! static package: &'static [(&'static [u8], &'static [u8])] = resources_package!([
//!     "path/to/resources",
//!     "other/path/to/other/resources"
//! ]);
//! # fn main() {}
//! ```
//! 
//! The type of the static variable is a slice of (`filename`, `content`). `filename` is
//!  the result of calling `Path::as_vec()` where the path is relative to the specified directory.
//! To turn it back into a path, call `Path::new(filename)`.
//!
//! **Important**: because of technical reasons, the crate will produce POSIX path if you compile
//!  on POSIX, and Windows path if you compile on Windows. Take care if you send them over the
//!  network.
//!

#![feature(plugin_registrar)]
#![feature(quote)]

extern crate rustc;
extern crate syntax;

use std::io::fs::{mod, PathExtensions};
use std::rc::Rc;
use syntax::ast::{mod, TokenTree};
use syntax::ext::build::AstBuilder;
use syntax::ext::base::{mod, DummyResult, ExtCtxt, MacResult};
use syntax::codemap::Span;
use syntax::ptr::P;

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
                vec![match base::expr_to_string(ecx, parameters.as_slice().get(0).unwrap().clone(),
                    "expected string literal")
                    {
                        Some((s, _)) => Path::new(s.get().to_string()),
                        None => return DummyResult::any(span)
                    }
                ]
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

    // building the list of elements
    let data: Vec<P<ast::Expr>> = parameters
        .into_iter()
        .map(|p| {
            // turning each element into an absolute path
            std::os::make_absolute(&base_path.join(p))
        })
        .flat_map(|path| {
            // for each element, returning a iterator of (Path, Path) where the first one
            //  is a real file and the second one is the original requested directory
            match fs::walk_dir(&path) {
                Ok(val) => val,
                Err(err) => {
                    ecx.span_err(span, format!("error while reading the content of `{}`: {}",
                        path.display(), err).as_slice());
                    fail!();    // no better solution T_T
                }
            }.zip(std::iter::iterate(path, |v| v))
        })
        .map(|(path, base)| {
            // turning this into a (Path, Path) where the first one is the name of the resource
            //  and the second one is the absolute path on the disk
            (path.path_relative_from(&base).unwrap(), path.clone())
        })
        .filter_map(|(name, path)| {
            if !path.is_file() {
                return None;
            }

            // adding a compilation dependency to the file, so that a recompilation will be
            //  triggered if the file is modified
            ecx.codemap().new_filemap(path.as_str().unwrap().to_string(), "".to_string());

            // getting the content of the file as an include_bin! expression
            let content = {
                let path = path.as_str().unwrap();
                quote_expr!(ecx, include_bin!($path))
            };

            // returning the tuple in the array of resources
            Some(ecx.expr_tuple(span.clone(), vec![
                ecx.expr_lit(span.clone(), ast::LitBinary(Rc::new(name.into_vec()))),
                content
            ]))
        })
        .collect();

    // including data
    base::MacExpr::new(ecx.expr_vec_slice(span.clone(), data))
}
