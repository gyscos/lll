use std::path;

use crate::commands::{CommandLine, LllCommand, LllRunnable};
use crate::context::LllContext;
use crate::error::LllError;
use crate::window::LllView;

use rustyline::completion::{escape, Quote};

#[cfg(unix)]
static DEFAULT_BREAK_CHARS: [u8; 18] = [
    b' ', b'\t', b'\n', b'"', b'\\', b'\'', b'`', b'@', b'$', b'>', b'<', b'=', b';', b'|', b'&',
    b'{', b'(', b'\0',
];
#[cfg(unix)]
static ESCAPE_CHAR: Option<char> = Some('\\');

#[derive(Clone, Debug)]
pub struct RenameFile {
    path: path::PathBuf,
}

impl RenameFile {
    pub fn new(path: path::PathBuf) -> Self {
        RenameFile { path }
    }
    pub const fn command() -> &'static str {
        "rename"
    }

    pub fn rename_file(
        &self,
        path: &path::PathBuf,
        context: &mut LllContext,
        view: &LllView,
    ) -> std::io::Result<()> {
        let new_path = &self.path;
        if new_path.exists() {
            let err =
                std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Filename already exists");
            return Err(err);
        }
        std::fs::rename(&path, &new_path)?;
        let curr_tab = &mut context.tabs[context.curr_tab_index];
        curr_tab
            .curr_list
            .update_contents(&context.config_t.sort_option)?;
        curr_tab.refresh_curr(&view.mid_win, &context.config_t);
        Ok(())
    }
}

impl LllCommand for RenameFile {}

impl std::fmt::Display for RenameFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Self::command())
    }
}

impl LllRunnable for RenameFile {
    fn execute(&self, context: &mut LllContext, view: &LllView) -> Result<(), LllError> {
        let mut path: Option<path::PathBuf> = None;

        let curr_list = &context.tabs[context.curr_tab_index].curr_list;
        if let Some(s) = curr_list.get_curr_ref() {
            path = Some(s.file_path().clone());
        }

        if let Some(path) = path {
            match self.rename_file(&path, context, view) {
                Ok(_) => {}
                Err(e) => return Err(LllError::IO(e)),
            }
            ncurses::doupdate();
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RenameFileAppend;

impl RenameFileAppend {
    pub fn new() -> Self {
        RenameFileAppend {}
    }
    pub const fn command() -> &'static str {
        "rename_append"
    }

    pub fn rename_file(
        &self,
        context: &mut LllContext,
        view: &LllView,
        file_name: String,
    ) -> Result<(), LllError> {
        let prefix;
        let suffix;
        if let Some(ext) = file_name.rfind('.') {
            prefix = format!("rename {}", &file_name[0..ext]);
            suffix = String::from(&file_name[ext..]);
        } else {
            prefix = format!("rename {}", file_name);
            suffix = String::new();
        }

        let command = CommandLine::new(prefix, suffix);
        command.readline(context, view)
    }
}

impl LllCommand for RenameFileAppend {}

impl std::fmt::Display for RenameFileAppend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Self::command())
    }
}

impl LllRunnable for RenameFileAppend {
    fn execute(&self, context: &mut LllContext, view: &LllView) -> Result<(), LllError> {
        let curr_list = &context.tabs[context.curr_tab_index].curr_list;
        let file_name = match curr_list.get_curr_ref() {
            Some(s) => {
                let escaped = escape(
                    String::from(s.file_name()),
                    ESCAPE_CHAR,
                    &DEFAULT_BREAK_CHARS,
                    Quote::None,
                );
                Some(escaped)
            }
            None => None,
        };

        if let Some(file_name) = file_name {
            self.rename_file(context, view, file_name)?;
            ncurses::doupdate();
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RenameFilePrepend;

impl RenameFilePrepend {
    pub fn new() -> Self {
        RenameFilePrepend {}
    }
    pub const fn command() -> &'static str {
        "rename_prepend"
    }

    pub fn rename_file(
        &self,
        context: &mut LllContext,
        view: &LllView,
        file_name: String,
    ) -> Result<(), LllError> {
        let prefix = String::from("rename ");
        let suffix = file_name;

        let command = CommandLine::new(prefix, suffix);
        command.readline(context, view)
    }
}

impl LllCommand for RenameFilePrepend {}

impl std::fmt::Display for RenameFilePrepend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Self::command())
    }
}

impl LllRunnable for RenameFilePrepend {
    fn execute(&self, context: &mut LllContext, view: &LllView) -> Result<(), LllError> {
        let curr_list = &context.tabs[context.curr_tab_index].curr_list;
        let file_name = match curr_list.get_curr_ref() {
            Some(s) => {
                let escaped = escape(
                    String::from(s.file_name()),
                    ESCAPE_CHAR,
                    &DEFAULT_BREAK_CHARS,
                    Quote::None,
                );
                Some(escaped)
            }
            None => None,
        };

        if let Some(file_name) = file_name {
            self.rename_file(context, view, file_name)?;
            ncurses::doupdate();
        }
        Ok(())
    }
}
