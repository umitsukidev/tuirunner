use crate::runner::TaskRunner;

pub struct Store<'a> {
    pub runner: &'a TaskRunner,
}
