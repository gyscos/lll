use std::collections::{hash_map::Entry, HashMap};
use std::path::{Path, PathBuf};

use crate::fs::LllDirList;
use crate::sort;

pub trait DirectoryHistory {
    fn populate_to_root(
        &mut self,
        pathbuf: &PathBuf,
        sort_option: &sort::SortOption,
    ) -> std::io::Result<()>;
    fn pop_or_create(
        &mut self,
        path: &Path,
        sort_option: &sort::SortOption,
    ) -> std::io::Result<LllDirList>;
    fn get_mut_or_create(
        &mut self,
        path: &Path,
        sort_option: &sort::SortOption,
    ) -> std::io::Result<&mut LllDirList>;
    fn depreciate_all_entries(&mut self);
}

pub type LllHistory = HashMap<PathBuf, LllDirList>;

impl DirectoryHistory for LllHistory {
    fn populate_to_root(
        &mut self,
        pathbuf: &PathBuf,
        sort_option: &sort::SortOption,
    ) -> std::io::Result<()> {
        let mut ancestors = pathbuf.ancestors();
        match ancestors.next() {
            None => {}
            Some(mut ancestor) => {
                for curr in ancestors {
                    let mut dirlist = LllDirList::new(curr.to_path_buf().clone(), sort_option)?;
                    let index = dirlist.contents.iter().enumerate().find_map(|(i, dir)| {
                        if dir.file_path() == ancestor {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    if let Some(i) = index {
                        dirlist.index = Some(i);
                    }
                    self.insert(curr.to_path_buf(), dirlist);
                    ancestor = curr;
                }
            }
        }
        Ok(())
    }

    fn pop_or_create(
        &mut self,
        path: &Path,
        sort_option: &sort::SortOption,
    ) -> std::io::Result<LllDirList> {
        match self.remove(&path.to_path_buf()) {
            Some(mut dirlist) => {
                if dirlist.need_update() {
                    dirlist.update_contents(&sort_option)?
                } else {
                    let metadata = std::fs::symlink_metadata(dirlist.file_path())?;

                    let modified = metadata.modified()?;
                    if modified > dirlist.metadata.modified {
                        dirlist.update_contents(&sort_option)?
                    }
                }
                Ok(dirlist)
            }
            None => {
                let path_clone = path.to_path_buf();
                LllDirList::new(path_clone, &sort_option)
            }
        }
    }
    fn get_mut_or_create(
        &mut self,
        path: &Path,
        sort_option: &sort::SortOption,
    ) -> std::io::Result<&mut LllDirList> {
        match self.entry(path.to_path_buf().clone()) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let s = LllDirList::new(path.to_path_buf(), &sort_option)?;
                Ok(entry.insert(s))
            }
        }
    }

    fn depreciate_all_entries(&mut self) {
        self.iter_mut().for_each(|(_, v)| v.depreciate());
    }
}
