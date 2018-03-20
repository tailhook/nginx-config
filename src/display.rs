use ast;
use std::fmt;
use format::{Displayable, Formatter, Style};

impl Displayable for ast::Main {
    fn display(&self, f: &mut Formatter) {
        for item in &self.directives {
            item.display(f);
        }
    }
}

impl Displayable for ast::Directive {
    fn display(&self, f: &mut Formatter) {
        self.item.display(f)
    }
}

impl Displayable for ast::Item {
    fn display(&self, f: &mut Formatter) {
        use ast::Item::*;
        f.indent();
        match *self {
            Daemon(opt) => {
                f.write("daemon ");
                f.write(if opt { "on" } else { "off" });
                f.end();
            }
            MasterProcess(opt) => {
                f.write("master_process ");
                f.write(if opt { "on" } else { "off" });
                f.end();
            }
            WorkerProcesses(ast::WorkerProcesses::Auto) => {
                f.write("worker_processes auto");
                f.end();
            }
            WorkerProcesses(ast::WorkerProcesses::Exact(n)) => {
                f.write("worker_processes ");
                f.write(&format!("{}", n));
                f.end();
            }
        }
    }
}

fn to_string<T: Displayable>(v: &T) -> String {
    let style = Style::default();
    let mut formatter = Formatter::new(&style);
    v.display(&mut formatter);
    formatter.into_string()
}

macro_rules! impl_display {
    ($( $typ: ty, )+) => {
        $(
            impl fmt::Display for $typ {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str(&to_string(self))
                }
            }
        )+
    };
}
impl_display!(
    ast::Main,
);
