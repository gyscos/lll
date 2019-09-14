use crate::ui;
use crate::window::LllPanel;

#[derive(Debug)]
pub struct LllView {
    pub top_win: LllPanel,
    pub tab_win: LllPanel,
    pub left_win: LllPanel,
    pub mid_win: LllPanel,
    pub right_win: LllPanel,
    pub bot_win: LllPanel,
    pub win_ratio: (usize, usize, usize),
}

impl LllView {
    pub fn new(win_ratio: (usize, usize, usize)) -> Self {
        let sum_ratio: usize = win_ratio.0 + win_ratio.1 + win_ratio.2;

        let (term_rows, term_cols) = ui::getmaxyx();
        let term_divide: i32 = term_cols / sum_ratio as i32;

        // window for tabs
        let win_xy: (i32, i32) = (1, 10);
        let win_coord: (usize, usize) = (0, term_cols as usize - win_xy.1 as usize);
        let tab_win = LllPanel::new(win_xy.0, win_xy.1, win_coord);

        let win_xy: (i32, i32) = (1, term_cols - tab_win.cols);
        let win_coord: (usize, usize) = (0, 0);
        let top_win = LllPanel::new(win_xy.0, win_xy.1, win_coord);

        let offset = 0;

        let win_xy: (i32, i32) = (term_rows - 2, (term_divide * win_ratio.0 as i32) - 1);
        let win_coord: (usize, usize) = (1, offset);
        let left_win = LllPanel::new(win_xy.0, win_xy.1, win_coord);

        let offset = offset + win_ratio.0;

        let win_xy: (i32, i32) = (term_rows - 2, (term_divide * win_ratio.1 as i32) - 1);
        let win_coord: (usize, usize) = (1, term_divide as usize * offset);
        let mid_win = LllPanel::new(win_xy.0, win_xy.1, win_coord);

        let offset = offset + win_ratio.1;

        let win_xy: (i32, i32) = (term_rows - 2, term_cols - (term_divide * offset as i32) - 1);
        let win_coord: (usize, usize) = (1, term_divide as usize * offset);
        let right_win = LllPanel::new(win_xy.0, win_xy.1, win_coord);

        let win_xy: (i32, i32) = (1, term_cols);
        let win_coord: (usize, usize) = (term_rows as usize - 1, 0);
        let bot_win = LllPanel::new(win_xy.0, win_xy.1, win_coord);

        LllView {
            top_win,
            tab_win,
            left_win,
            mid_win,
            right_win,
            bot_win,
            win_ratio,
        }
    }

    pub fn resize_views(&mut self) {
        let new_view = Self::new(self.win_ratio);

        self.top_win = new_view.top_win;
        self.bot_win = new_view.bot_win;
        self.tab_win = new_view.tab_win;
        self.left_win = new_view.left_win;
        self.mid_win = new_view.mid_win;
        self.right_win = new_view.right_win;
    }
}
