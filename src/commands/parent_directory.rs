use crate::commands::{LllCommand, LllRunnable};
use crate::context::LllContext;
use crate::error::LllError;
use crate::history::DirectoryHistory;
use crate::window::LllView;

#[derive(Clone, Debug)]
pub struct ParentDirectory;

impl ParentDirectory {
    pub fn new() -> Self {
        ParentDirectory
    }
    pub const fn command() -> &'static str {
        "parent_directory"
    }

    pub fn parent_directory(
        context: &mut LllContext,
        view: &LllView,
    ) -> Result<(), std::io::Error> {
        let curr_tab = &mut context.tabs[context.curr_tab_index];
        if !curr_tab.curr_path.pop() {
            return Ok(());
        }
        std::env::set_current_dir(&curr_tab.curr_path)?;

        let mut new_curr_list = curr_tab
            .history
            .pop_or_create(&curr_tab.curr_path, &context.config_t.sort_option)?;

        std::mem::swap(&mut curr_tab.curr_list, &mut new_curr_list);
        curr_tab
            .history
            .insert(new_curr_list.file_path().clone(), new_curr_list);

        curr_tab.refresh(view, &context.config_t);
        ncurses::doupdate();
        Ok(())
    }
}

impl LllCommand for ParentDirectory {}

impl std::fmt::Display for ParentDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(Self::command())
    }
}

impl LllRunnable for ParentDirectory {
    fn execute(&self, context: &mut LllContext, view: &LllView) -> Result<(), LllError> {
        match Self::parent_directory(context, view) {
            Ok(_) => Ok(()),
            Err(e) => Err(LllError::IO(e)),
        }
    }
}
