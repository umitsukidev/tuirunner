use crate::{config::Task, runner::TaskStatus};
use std::collections::HashMap;

pub struct Store<'a> {
    pub tasks: &'a HashMap<String, Task>,
    pub task_statuses: &'a HashMap<String, TaskStatus>,
    pub visible_tasks: &'a [String],
}
