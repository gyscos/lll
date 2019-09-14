use std::{fs, path};

use crate::fs::{LllDirEntry, LllMetadata};
use crate::sort;
use crate::window::LllPageState;

#[derive(Debug)]
pub struct LllDirList {
    pub index: Option<usize>,
    path: path::PathBuf,
    outdated: bool,
    pub metadata: LllMetadata,
    pub contents: Vec<LllDirEntry>,
    pub pagestate: LllPageState,
}

impl LllDirList {
    pub fn new(path: path::PathBuf, sort_option: &sort::SortOption) -> std::io::Result<Self> {
        let mut contents = read_dir_list(path.as_path(), sort_option)?;
        contents.sort_by(&sort_option.compare_func());

        let index = if contents.is_empty() { None } else { Some(0) };

        let metadata = LllMetadata::from(&path)?;
        let pagestate = LllPageState::default();

        Ok(LllDirList {
            index,
            path,
            outdated: false,
            metadata,
            contents,
            pagestate,
        })
    }

    pub fn depreciate(&mut self) {
        self.outdated = true;
    }

    pub fn need_update(&self) -> bool {
        self.outdated
    }

    pub fn file_path(&self) -> &path::PathBuf {
        &self.path
    }

    pub fn update_contents(&mut self, sort_option: &sort::SortOption) -> std::io::Result<()> {
        let sort_func = sort_option.compare_func();
        let mut contents = read_dir_list(&self.path, sort_option)?;
        contents.sort_by(&sort_func);

        let contents_len = contents.len();
        if contents_len == 0 {
            self.index = None;
        } else {
            self.index = match self.index {
                Some(index) => {
                    if index >= contents_len {
                        Some(contents_len - 1)
                    } else {
                        self.index
                    }
                }
                None => Some(0),
            };
        }

        let metadata = LllMetadata::from(&self.path)?;
        self.metadata = metadata;
        self.contents = contents;
        self.outdated = false;

        Ok(())
    }

    pub fn selected_entries(&self) -> impl Iterator<Item = &LllDirEntry> {
        self.contents.iter().filter(|entry| entry.is_selected())
    }

    pub fn get_selected_paths(&self) -> Vec<&path::PathBuf> {
        let vec: Vec<&path::PathBuf> = self.selected_entries().map(|e| e.file_path()).collect();
        if vec.is_empty() {
            match self.get_curr_ref() {
                Some(s) => vec![s.file_path()],
                _ => vec![],
            }
        } else {
            vec
        }
    }

    pub fn get_curr_ref(&self) -> Option<&LllDirEntry> {
        self.get_curr_ref_(self.index?)
    }

    pub fn get_curr_mut(&mut self) -> Option<&mut LllDirEntry> {
        self.get_curr_mut_(self.index?)
    }

    fn get_curr_mut_(&mut self, index: usize) -> Option<&mut LllDirEntry> {
        if index < self.contents.len() {
            Some(&mut self.contents[index])
        } else {
            None
        }
    }

    fn get_curr_ref_(&self, index: usize) -> Option<&LllDirEntry> {
        if index < self.contents.len() {
            Some(&self.contents[index])
        } else {
            None
        }
    }
}

fn read_dir_list(
    path: &path::Path,
    sort_option: &sort::SortOption,
) -> std::io::Result<Vec<LllDirEntry>> {
    let filter_func = sort_option.filter_func();
    let results: Vec<LllDirEntry> = fs::read_dir(path)?
        .filter(filter_func)
        .filter_map(map_entry_default)
        .collect();
    Ok(results)
}

fn map_entry_default(result: std::io::Result<fs::DirEntry>) -> Option<LllDirEntry> {
    match result {
        Ok(direntry) => match LllDirEntry::from(&direntry) {
            Ok(s) => Some(s),
            Err(_) => None,
        },
        Err(_) => None,
    }
}
