use std::path;

use crate::commands;

use crate::context::LllContext;
use crate::error::LllError;
use crate::history::DirectoryHistory;
use crate::window::LllView;
use commands::{LllCommand, LllRunnable};

#[derive(Clone, Debug)]
pub struct ChangeDirectory {
    path: path::PathBuf,
}

impl ChangeDirectory {
    pub fn new(path: path::PathBuf) -> Self {
        ChangeDirectory { path }
    }
    pub const fn command() -> &'static str {
        "cd"
    }

    pub fn change_directory(
        path: &path::PathBuf,
        context: &mut LllContext,
        view: &LllView,
    ) -> Result<(), std::io::Error> {
        let curr_tab = &mut context.tabs[context.curr_tab_index];

        std::env::set_current_dir(path.as_path())?;
        curr_tab.curr_path = path.clone();

        curr_tab
            .history
            .populate_to_root(&curr_tab.curr_path, &context.config_t.sort_option)?;

        let mut new_curr_list = curr_tab
            .history
            .pop_or_create(&curr_tab.curr_path, &context.config_t.sort_option)?;

        std::mem::swap(&mut curr_tab.curr_list, &mut new_curr_list);
        curr_tab
            .history
            .insert(new_curr_list.file_path().clone(), new_curr_list);

        curr_tab.refresh(view, &context.config_t);
        Ok(())
    }
}

impl LllCommand for ChangeDirectory {}

impl std::fmt::Display for ChangeDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", Self::command(), self.path.to_str().unwrap())
    }
}

impl LllRunnable for ChangeDirectory {
    fn execute(&self, context: &mut LllContext, view: &LllView) -> Result<(), LllError> {
        match Self::change_directory(&self.path, context, view) {
            Ok(_) => {}
            Err(e) => return Err(LllError::IO(e)),
        }
        ncurses::doupdate();
        Ok(())
    }
}
