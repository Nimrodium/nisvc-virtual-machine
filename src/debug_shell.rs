use rustyline::{self, history::FileHistory, DefaultEditor, Editor};

use crate::ExecutionError;
pub struct Shell {
    readline: Editor<(), FileHistory>,
}

impl Shell {
    pub fn new() -> Result<Self, ExecutionError> {
        let readline = DefaultEditor::new()
            .map_err(|e| ExecutionError::new(format!("failed to initialize debug shell: {e}")))?;
        Ok(Self { readline })
    }

    pub fn enter() -> Result<(), ExecutionError> {
        todo!()
    }

    fn prompt() -> Result<String, ExecutionError> {
        todo!()
    }
    fn exec_cmdline() -> Result<String, ExecutionError> {
        todo!()
    }
}
