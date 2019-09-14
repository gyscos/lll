use std::process;
use std::time;

use crate::commands::{CommandKeybind, FileOperationThread, LllCommand, ReloadDirList};
use crate::config::{self, LllCommandMapping, LllConfig};
use crate::context::LllContext;
use crate::error::LllError;
use crate::tab::LllTab;
use crate::ui;
use crate::window::LllPanel;
use crate::window::LllView;

fn recurse_get_keycommand(keymap: &LllCommandMapping) -> Option<&dyn LllCommand> {
    let (term_rows, term_cols) = ui::getmaxyx();
    ncurses::timeout(-1);

    let ch: i32 = {
        let keymap_len = keymap.len();
        let win = LllPanel::new(
            keymap_len as i32 + 1,
            term_cols,
            ((term_rows - keymap_len as i32 - 2) as usize, 0),
        );

        let mut display_vec: Vec<String> = keymap
            .iter()
            .map(|(k, v)| format!("  {}\t{}", *k as u8 as char, v))
            .collect();
        display_vec.sort();

        win.move_to_top();
        ui::display_menu(&win, &display_vec);
        ncurses::doupdate();

        ncurses::wgetch(win.win)
    };
    ncurses::doupdate();

    if ch == config::keymap::ESCAPE {
        None
    } else {
        match keymap.get(&ch) {
            Some(CommandKeybind::CompositeKeybind(m)) => recurse_get_keycommand(&m),
            Some(CommandKeybind::SimpleKeybind(s)) => Some(s.as_ref()),
            _ => None,
        }
    }
}

fn reload_tab(index: usize, context: &mut LllContext, view: &LllView) -> std::io::Result<()> {
    ReloadDirList::reload(index, context)?;
    if index == context.curr_tab_index {
        let dirty_tab = &mut context.tabs[index];
        dirty_tab.refresh(view, &context.config_t);
    }
    Ok(())
}

fn join_thread(
    context: &mut LllContext,
    thread: FileOperationThread<u64, fs_extra::TransitProcess>,
    view: &LllView,
) -> std::io::Result<()> {
    ncurses::werase(view.bot_win.win);
    ncurses::doupdate();

    let (tab_src, tab_dest) = (thread.tab_src, thread.tab_dest);
    match thread.handle.join() {
        Err(e) => {
            ui::wprint_err(&view.bot_win, format!("{:?}", e).as_str());
            view.bot_win.queue_for_refresh();
        }
        Ok(_) => {
            if tab_src < context.tabs.len() {
                reload_tab(tab_src, context, view)?;
            }
            if tab_dest != tab_src && tab_dest < context.tabs.len() {
                reload_tab(tab_dest, context, view)?;
            }
        }
    }
    Ok(())
}

fn process_threads(context: &mut LllContext, view: &LllView) -> std::io::Result<()> {
    let thread_wait_duration: time::Duration = time::Duration::from_millis(100);
    for i in 0..context.threads.len() {
        match &context.threads[i].recv_timeout(&thread_wait_duration) {
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let thread = context.threads.swap_remove(i);
                join_thread(context, thread, view)?;
                ncurses::doupdate();
                break;
            }
            Ok(progress_info) => {
                ui::show_fs_operation_progress(&view.bot_win, &progress_info);
                ncurses::doupdate();
            }
            _ => {}
        }
    }
    Ok(())
}

#[inline]
fn resize_handler(context: &mut LllContext, view: &LllView) {
    ui::redraw_tab_view(&view.tab_win, &context);

    let curr_tab = &mut context.tabs[context.curr_tab_index];
    curr_tab.refresh(view, &context.config_t);
    ncurses::doupdate();
}

fn init_context(context: &mut LllContext, view: &LllView) {
    match std::env::current_dir() {
        Ok(curr_path) => match LllTab::new(curr_path, &context.config_t.sort_option) {
            Ok(tab) => {
                context.tabs.push(tab);
                context.curr_tab_index = context.tabs.len() - 1;

                ui::redraw_tab_view(&view.tab_win, &context);
                let curr_tab = &mut context.tabs[context.curr_tab_index];
                curr_tab.refresh(view, &context.config_t);
                ncurses::doupdate();
            }
            Err(e) => {
                ui::end_ncurses();
                eprintln!("{}", e);
                process::exit(1);
            }
        },
        Err(e) => {
            ui::end_ncurses();
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

pub fn run(config_t: LllConfig, keymap_t: LllCommandMapping) {
    ui::init_ncurses();

    let mut context = LllContext::new(config_t);
    let mut view = LllView::new(context.config_t.column_ratio);
    init_context(&mut context, &view);

    while !context.exit {
        if !context.threads.is_empty() {
            ncurses::timeout(0);
            match process_threads(&mut context, &view) {
                Ok(_) => {}
                Err(e) => ui::wprint_err(&view.bot_win, e.to_string().as_str()),
            }
            ncurses::doupdate();
        } else {
            ncurses::timeout(-1);
        }

        if let Some(ch) = ncurses::get_wch() {
            let ch = match ch {
                ncurses::WchResult::Char(s) => s as i32,
                ncurses::WchResult::KeyCode(s) => s,
            };

            if ch == ncurses::KEY_RESIZE {
                view.resize_views();
                resize_handler(&mut context, &view);
                continue;
            }

            let keycommand;

            match keymap_t.get(&ch) {
                Some(CommandKeybind::CompositeKeybind(m)) => match recurse_get_keycommand(&m) {
                    Some(s) => keycommand = s,
                    None => continue,
                },
                Some(CommandKeybind::SimpleKeybind(s)) => {
                    keycommand = s.as_ref();
                }
                None => {
                    // TODO: remove this eventually
                    ui::wprint_err(&view.bot_win, &format!("Unknown keycode: {}", ch));
                    ncurses::doupdate();
                    continue;
                }
            }
            match keycommand.execute(&mut context, &view) {
                Ok(()) => {}
                Err(LllError::IO(e)) => {
                    ui::wprint_err(&view.bot_win, e.to_string().as_str());
                    ncurses::doupdate();
                }
                Err(LllError::Keymap(e)) => {
                    ui::wprint_err(&view.bot_win, e.to_string().as_str());
                    ncurses::doupdate();
                }
            }
        }
    }
    ui::end_ncurses();
}
