use std::env;

use crate::commands::{LllCommand, LllRunnable};
use crate::context::LllContext;
use crate::error::LllError;
use crate::ui;
use crate::window::LllView;

#[derive(Clone, Debug)]
pub struct TabSwitch {
    movement: i32,
}

impl TabSwitch {
    pub fn new(movement: i32) -> Self {
        TabSwitch { movement }
    }
    pub const fn command() -> &'static str {
        "tab_switch"
    }

    pub fn tab_switch(
        new_index: usize,
        context: &mut LllContext,
        view: &LllView,
    ) -> std::io::Result<()> {
        context.curr_tab_index = new_index;
        let path = &context.curr_tab_ref().curr_path;
        env::set_current_dir(path)?;
        {
            let curr_tab = &mut context.tabs[context.curr_tab_index];
            if curr_tab.curr_list.need_update() {
                curr_tab
                    .curr_list
                    .update_contents(&context.config_t.sort_option)?;
            }
            curr_tab.refresh(view, &context.config_t);
        }
        ui::redraw_tab_view(&view.tab_win, &context);
        ncurses::doupdate();
        Ok(())
    }
}

impl LllCommand for TabSwitch {}

impl std::fmt::Display for TabSwitch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", Self::command(), self.movement)
    }
}

impl LllRunnable for TabSwitch {
    fn execute(&self, context: &mut LllContext, view: &LllView) -> Result<(), LllError> {
        let mut new_index = context.curr_tab_index as i32 + self.movement;
        let tab_len = context.tabs.len() as i32;
        while new_index < 0 {
            new_index += tab_len;
        }
        while new_index >= tab_len {
            new_index -= tab_len;
        }
        let new_index = new_index as usize;
        match Self::tab_switch(new_index, context, view) {
            Ok(_) => Ok(()),
            Err(e) => Err(LllError::IO(e)),
        }
    }
}
