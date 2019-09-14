mod change_directory;
mod command_line;
mod cursor_move;
mod delete_files;
mod file_operations;
mod new_directory;
mod open_file;
mod parent_directory;
mod quit;
mod reload_dir;
mod rename_file;
mod search;
mod selection;
mod set_mode;
mod show_hidden;
mod tab_switch;

pub use self::change_directory::ChangeDirectory;
pub use self::command_line::CommandLine;
pub use self::cursor_move::{
    CursorMoveDown, CursorMoveEnd, CursorMoveHome, CursorMovePageDown, CursorMovePageUp,
    CursorMoveUp,
};
pub use self::delete_files::DeleteFiles;
pub use self::file_operations::{CopyFiles, CutFiles, FileOperationThread, PasteFiles};
pub use self::new_directory::NewDirectory;
pub use self::open_file::{OpenFile, OpenFileWith};
pub use self::parent_directory::ParentDirectory;
pub use self::quit::ForceQuit;
pub use self::quit::Quit;
pub use self::reload_dir::ReloadDirList;
pub use self::rename_file::{RenameFile, RenameFileAppend, RenameFilePrepend};
pub use self::search::{Search, SearchNext, SearchPrev};
pub use self::selection::SelectFiles;
pub use self::set_mode::SetMode;
pub use self::show_hidden::ToggleHiddenFiles;
pub use self::tab_switch::TabSwitch;

use std::path::PathBuf;

use crate::config::LllCommandMapping;
use crate::context::LllContext;
use crate::error::{KeymapError, LllError};
use crate::window::LllView;

use crate::HOME_DIR;

#[derive(Debug)]
pub enum CommandKeybind {
    SimpleKeybind(Box<dyn LllCommand>),
    CompositeKeybind(LllCommandMapping),
}

pub trait LllRunnable {
    fn execute(&self, context: &mut LllContext, view: &LllView) -> Result<(), LllError>;
}

pub trait LllCommand: LllRunnable + std::fmt::Display + std::fmt::Debug {}

impl std::fmt::Display for CommandKeybind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommandKeybind::SimpleKeybind(s) => write!(f, "{}", s),
            CommandKeybind::CompositeKeybind(_) => write!(f, "..."),
        }
    }
}

pub fn from_args(command: &str, args: &[&str]) -> Result<Box<dyn LllCommand>, KeymapError> {
    match command {
        "cd" => match args.len() {
            0 => match HOME_DIR.as_ref() {
                Some(s) => Ok(Box::new(self::ChangeDirectory::new(s.clone()))),
                None => Err(KeymapError::new(
                    Some("cd"),
                    String::from("Cannot find home directory"),
                )),
            },
            1 => match args[0] {
                ".." => Ok(Box::new(self::ParentDirectory::new())),
                arg => Ok(Box::new(self::ChangeDirectory::new(PathBuf::from(arg)))),
            },
            i => Err(KeymapError::new(
                Some("cd"),
                format!("Expected 1 argument, got {}", i),
            )),
        },
        "copy_files" => Ok(Box::new(self::CopyFiles::new())),
        "console" => match args.len() {
            0 => Ok(Box::new(self::CommandLine::new(
                String::new(),
                String::new(),
            ))),
            1 => Ok(Box::new(self::CommandLine::new(
                String::from(args[0]),
                String::new(),
            ))),
            i => Err(KeymapError::new(
                Some("console"),
                format!("Expected 0 or 2 arguments, got {}", i),
            )),
        },
        "cursor_move_down" => match args.len() {
            0 => Ok(Box::new(self::CursorMoveDown::new(1))),
            1 => match args[0].parse::<usize>() {
                Ok(s) => Ok(Box::new(self::CursorMoveDown::new(s))),
                Err(e) => Err(KeymapError::new(Some("cursor_move_down"), e.to_string())),
            },
            i => Err(KeymapError::new(
                Some("cursor_move_down"),
                format!("Expected 0 or 1 arguments, got {}", i),
            )),
        },
        "cursor_move_up" => match args.len() {
            0 => Ok(Box::new(self::CursorMoveUp::new(1))),
            1 => match args[0].parse::<usize>() {
                Ok(s) => Ok(Box::new(self::CursorMoveUp::new(s))),
                Err(e) => Err(KeymapError::new(Some("cursor_move_down"), e.to_string())),
            },
            i => Err(KeymapError::new(
                Some("cursor_move_down"),
                format!("Expected 0 or 1 arguments, got {}", i),
            )),
        },
        "cursor_move_home" => Ok(Box::new(self::CursorMoveHome::new())),
        "cursor_move_end" => Ok(Box::new(self::CursorMoveEnd::new())),
        "cursor_move_page_up" => Ok(Box::new(self::CursorMovePageUp::new())),
        "cursor_move_page_down" => Ok(Box::new(self::CursorMovePageDown::new())),
        "cut_files" => Ok(Box::new(self::CutFiles::new())),
        "delete_files" => Ok(Box::new(self::DeleteFiles::new())),
        "force_quit" => Ok(Box::new(self::ForceQuit::new())),
        "mkdir" => {
            if args.is_empty() {
                Err(KeymapError::new(
                    Some("mkdir"),
                    String::from("mkdir requires additional parameter"),
                ))
            } else {
                let paths: Vec<PathBuf> = args.iter().map(PathBuf::from).collect();
                Ok(Box::new(self::NewDirectory::new(paths)))
            }
        }
        "open_file" => Ok(Box::new(self::OpenFile::new())),
        "open_file_with" => Ok(Box::new(self::OpenFileWith::new())),
        "paste_files" => {
            let mut options = fs_extra::dir::CopyOptions::new();
            options.buffer_size = 1024 * 1024 * 4;
            for arg in args {
                match *arg {
                    "--overwrite" => options.overwrite = true,
                    "--skip_exist" => options.skip_exist = true,
                    _ => {
                        return Err(KeymapError::new(
                            Some("paste_files"),
                            format!("unknown option {}", arg),
                        ));
                    }
                }
            }
            Ok(Box::new(self::PasteFiles::new(options)))
        }
        "quit" => Ok(Box::new(self::Quit::new())),
        "reload_dir_list" => Ok(Box::new(self::ReloadDirList::new())),
        "rename" => match args.len() {
            1 => {
                let path: PathBuf = PathBuf::from(args[0]);
                Ok(Box::new(self::RenameFile::new(path)))
            }
            i => Err(KeymapError::new(
                Some("rename_file"),
                format!("Expected 1, got {}", i),
            )),
        },
        "rename_append" => Ok(Box::new(self::RenameFileAppend::new())),
        "rename_prepend" => Ok(Box::new(self::RenameFilePrepend::new())),
        "search" => match args.len() {
            1 => Ok(Box::new(self::Search::new(args[0]))),
            i => Err(KeymapError::new(
                Some("search"),
                format!("Expected 1, got {}", i),
            )),
        },
        "search_next" => Ok(Box::new(self::SearchNext::new())),
        "search_prev" => Ok(Box::new(self::SearchPrev::new())),
        "select_files" => {
            let mut toggle = false;
            let mut all = false;
            for arg in args {
                match *arg {
                    "--toggle" => toggle = true,
                    "--all" => all = true,
                    _ => {
                        return Err(KeymapError::new(
                            Some("select_files"),
                            format!("unknown option {}", arg),
                        ));
                    }
                }
            }
            Ok(Box::new(self::SelectFiles::new(toggle, all)))
        }
        "set_mode" => Ok(Box::new(self::SetMode::new())),
        "tab_switch" => {
            if args.len() == 1 {
                match args[0].parse::<i32>() {
                    Ok(s) => Ok(Box::new(self::TabSwitch::new(s))),
                    Err(e) => Err(KeymapError::new(Some("tab_switch"), e.to_string())),
                }
            } else {
                Err(KeymapError::new(
                    Some("tab_switch"),
                    String::from("No option provided"),
                ))
            }
        }
        "toggle_hidden" => Ok(Box::new(self::ToggleHiddenFiles::new())),
        inp => Err(KeymapError::new(None, format!("Unknown command: {}", inp))),
    }
}
