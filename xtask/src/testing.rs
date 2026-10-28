//! Shared test runner; no subprocess fixtures or global environment mutation.

use std::{cell::RefCell, collections::VecDeque, io};

use crate::cmd::{Command, Output, Runner};

#[derive(Default)]
pub(crate) struct RecordingRunner {
    pub commands: RefCell<Vec<(Command, bool)>>,
    pub results: RefCell<VecDeque<io::Result<Output>>>,
    pub dry_run: bool,
}

impl RecordingRunner {
    pub fn stdout(stdout: &[u8]) -> Self {
        let runner = Self::default();
        runner.results.borrow_mut().push_back(Ok(Output {
            stdout: stdout.to_vec(),
            ..Output::default()
        }));
        runner
    }

    pub fn assert_args(&self, index: usize, expected: &[&str]) {
        assert_eq!(
            self.commands.borrow()[index]
                .0
                .arguments()
                .collect::<Vec<_>>(),
            expected
        );
    }
}

impl Runner for RecordingRunner {
    fn execute(&self, command: &Command, capture: bool) -> io::Result<Output> {
        self.commands.borrow_mut().push((command.clone(), capture));
        self.results
            .borrow_mut()
            .pop_front()
            .expect("unexpected command")
    }

    fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}
