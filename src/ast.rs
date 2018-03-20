use position::Pos;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Main {
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    pub position: Pos,
    pub item: Item,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerProcesses {
    Auto,
    Exact(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Daemon(bool),
    MasterProcess(bool),
    WorkerProcesses(WorkerProcesses),
}
