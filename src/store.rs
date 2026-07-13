use crate::runner::TaskRunner;

pub struct Store<'a> {
    pub runner: &'a TaskRunner,
    pub visible_tasks: &'a [String],
}
