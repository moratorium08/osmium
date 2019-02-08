use paging;

enum Type {
    User,
}

enum Status {
    Free,
    Running,
    Runnable,
    NotRunnable,
    Zonmie,
}
pub struct Id(u32);

pub struct Process<'a> {
    table: Option<&'a mut paging::PageTable>,
    id: Id,
    parent_id: Id,
    proc_type: Type,
    status: Status,
}

impl<'a> Process<'a> {
    fn new(id: Id) -> Process<'a> {
        Process {
            table: None,
            id,
            parent_id: id,
            proc_type: Type::User,
            status: Status::Free,
        }
    }
}
