use std::collections::VecDeque;

use ast::Directive;


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
    /// `config.directives()` instead.
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
