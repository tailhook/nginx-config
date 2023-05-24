//! Various visitors for working with AST
use std::collections::VecDeque;

use crate::ast::Directive;
use crate::value::Value;


/// A deep iterator over all directives in configuration file or a part of it
#[derive(Debug)]
pub struct DirectiveIter<'a> {
    cur: Option<&'a Directive>,
    queue: VecDeque<&'a Directive>,
}

impl<'a> DirectiveIter<'a> {
    /// Start depth-first iteration over config
    ///
    /// This is the only constructor so var. But usually you want to use
    /// [`config.all_directives()`] instead.
    ///
    /// [`config.all_directives()`]: ast/fn.all_directives.html
    pub fn depth_first(start: &[Directive]) -> DirectiveIter {
        DirectiveIter {
            cur: None,
            queue: start.iter().rev().collect()
        }
    }
}

impl<'a> Iterator for DirectiveIter<'a> {
    type Item = &'a Directive;
    fn next(&mut self) -> Option<&'a Directive> {
        match self.cur.take() {
            Some(dir) => {
                if let Some(ch) = dir.item.children() {
                    self.queue.extend(ch.iter().rev());
                }
            }
            None => {}
        }
        match self.queue.pop_back() {
            Some(x) => {
                self.cur = Some(x);
                return Some(x);
            }
            None => None,
        }
    }
}

/// A recursive mutable depth-first visitor of directives
pub fn visit_mutable<F>(dirs: &mut Vec<Directive>, mut f: F)
    where F: FnMut(&mut Directive)
{
    _visit_mutable(dirs, &mut f)
}

fn _visit_mutable<F>(dirs: &mut Vec<Directive>, f: &mut F)
    where F: FnMut(&mut Directive)
{
    for dir in dirs {
        f(dir);
        match dir.item.children_mut() {
            Some(children) => _visit_mutable(children, f),
            None => {}
        }
    }
}

/// A recursive mutable visitor of all variable values
///
/// This function allows to replace all occurrences of some variable.
/// If `f` returns `None` variable is skipped.
pub fn replace_vars<'a, F, S>(dirs: &mut Vec<Directive>, mut f: F)
    where F: FnMut(&str) -> Option<S>,
          S: AsRef<str> + Into<String> + 'a,
{
    let mut inner_visitor = |val: &mut Value| {
        let ref mut f = f;
        val.replace_vars(f);
    };
    visit_mutable(dirs, |dir: &mut Directive| {
        let ref mut inner_visitor = inner_visitor;
        dir.visit_values_mut(inner_visitor);
    });
}
