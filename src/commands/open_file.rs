use std::path::{Path, PathBuf};

use crate::commands::{LllCommand, LllRunnable};
use crate::config::mimetype;
use crate::context::LllContext;
use crate::error::LllError;
use crate::history::DirectoryHistory;
use crate::textfield::LllTextField;
use crate::ui;
use crate::unix;
use crate::window;
use crate::window::LllView;

use crate::MIMETYPE_T;

#[derive(Clone, Debug)]
pub struct OpenFile;

impl OpenFile {
    pub fn new() -> Self {
        OpenFile
    }
    pub const fn command() -> &'static str {
        "open_file"
    }

    pub fn get_options<'a>(path: &Path) -> Vec<&'a mimetype::LllMimetypeEntry> {
        let mut mimetype_options: Vec<&mimetype::LllMimetypeEntry> = Vec::new();

        // extensions have priority
        if let Some(file_ext) = path.extension() {
            if let Some(file_ext) = file_ext.to_str() {
                let ext_entries = MIMETYPE_T.get_entries_for_ext(file_ext);
                mimetype_options.extend(ext_entries);
            }
        }
        mimetype_options
    }

    fn open(context: &mut LllContext, view: &LllView) -> Result<(), std::io::Error> {
        let mut path: Option<PathBuf> = None;
        {
            let curr_list = &context.tabs[context.curr_tab_index].curr_list;
            if let Some(entry) = curr_list.get_curr_ref() {
                if entry.file_path().is_dir() {
                    path = Some(entry.file_path().clone());
                }
            }
        }
        if let Some(path) = path {
            Self::open_directory(&path, context)?;
            let curr_tab = &mut context.tabs[context.curr_tab_index];
            if curr_tab.curr_list.need_update() {
                curr_tab
                    .curr_list
                    .update_contents(&context.config_t.sort_option)?;
            }
            curr_tab.refresh(view, &context.config_t);
        } else {
            let curr_tab = &context.tabs[context.curr_tab_index];
            let paths = curr_tab.curr_list.get_selected_paths();

            if paths.is_empty() {
                let err = std::io::Error::new(std::io::ErrorKind::NotFound, "No files selected");
                return Err(err);
            }
            Self::open_file(&paths);
            let curr_tab = &mut context.tabs[context.curr_tab_index];
            if curr_tab.curr_list.need_update() {
                curr_tab
                    .curr_list
                    .update_contents(&context.config_t.sort_option)?;
            }
            curr_tab.refresh(view, &context.config_t);
        }
        ncurses::doupdate();
        Ok(())
    }

    fn open_directory(path: &Path, context: &mut LllContext) -> std::io::Result<()> {
        std::env::set_current_dir(path)?;

        let curr_tab = &mut context.tabs[context.curr_tab_index];
        let mut new_curr_list = curr_tab
            .history
            .pop_or_create(path, &context.config_t.sort_option)?;

        std::mem::swap(&mut curr_tab.curr_list, &mut new_curr_list);
        curr_tab
            .history
            .insert(new_curr_list.file_path().clone(), new_curr_list);

        curr_tab.curr_path = path.to_path_buf().clone();
        Ok(())
    }

    fn open_file(paths: &[&PathBuf]) {
        let mimetype_options = Self::get_options(&paths[0]);

        ncurses::savetty();
        ncurses::endwin();
        if mimetype_options.is_empty() {
            open::that(&paths[0]).unwrap();
        } else {
            unix::open_with_entry(paths, &mimetype_options[0]);
        }
        ncurses::resetty();
        ncurses::refresh();
        ncurses::doupdate();
    }
}

impl LllCommand for OpenFile {}

impl std::fmt::Display for OpenFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(Self::command())
    }
}

impl LllRunnable for OpenFile {
    fn execute(&self, context: &mut LllContext, view: &LllView) -> Result<(), LllError> {
        match Self::open(context, view) {
            Ok(_) => Ok(()),
            Err(e) => Err(LllError::IO(e)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct OpenFileWith;

impl OpenFileWith {
    pub fn new() -> Self {
        OpenFileWith
    }
    pub const fn command() -> &'static str {
        "open_file_with"
    }

    pub fn open_with(paths: &[&PathBuf]) {
        const PROMPT: &str = ":open_with ";

        let mimetype_options: Vec<&mimetype::LllMimetypeEntry> = OpenFile::get_options(&paths[0]);
        let user_input: Option<String>;
        {
            let (term_rows, term_cols) = ui::getmaxyx();

            let option_size = mimetype_options.len();
            let display_win = window::LllPanel::new(
                option_size as i32 + 2,
                term_cols,
                (term_rows as usize - option_size - 2, 0),
            );

            let mut display_vec: Vec<String> = Vec::with_capacity(option_size);
            for (i, val) in mimetype_options.iter().enumerate() {
                display_vec.push(format!("  {}\t{}", i, val));
            }
            display_vec.sort();

            display_win.move_to_top();
            ui::display_menu(&display_win, &display_vec);
            ncurses::doupdate();

            let textfield =
                LllTextField::new(1, term_cols, (term_rows as usize - 1, 0), PROMPT, "", "");
            user_input = textfield.readline();
        }
        ncurses::doupdate();

        if let Some(user_input) = user_input {
            if user_input.is_empty() {
                return;
            }
            match user_input.parse::<usize>() {
                Ok(s) => {
                    if s < mimetype_options.len() {
                        ncurses::savetty();
                        ncurses::endwin();
                        unix::open_with_entry(paths, &mimetype_options[s]);
                        ncurses::resetty();
                        ncurses::refresh();
                    }
                }
                Err(_) => {
                    let args: Vec<String> =
                        user_input.split_whitespace().map(String::from).collect();
                    ncurses::savetty();
                    ncurses::endwin();
                    unix::open_with_args(paths, &args);
                    ncurses::resetty();
                    ncurses::refresh();
                }
            }
        }
    }
}

impl LllCommand for OpenFileWith {}

impl std::fmt::Display for OpenFileWith {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(Self::command())
    }
}

impl LllRunnable for OpenFileWith {
    fn execute(&self, context: &mut LllContext, _: &LllView) -> Result<(), LllError> {
        let curr_list = &context.tabs[context.curr_tab_index].curr_list;
        let paths = curr_list.get_selected_paths();
        Self::open_with(&paths);
        Ok(())
    }
}
