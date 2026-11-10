use crate::burst::render_burst_page;
use crate::edit::render_edit_page;
use crate::model::{Pages, UIState};
use crate::overview::render_overview;

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

pub fn main_view(ui: &mut UIState, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(10),
            Constraint::Percentage(90),
        ])
        .split(frame.area());

    let top = layout[0];
    let main = layout[1];

    frame.render_widget(
        Paragraph::new("Burstq").block(Block::new().borders(Borders::ALL)),
        top,
    );

    if let Some(page) = ui.current_page() {
        match page {
            Pages::Overview(overview_state) => {
                render_overview(frame, main, overview_state)
            }
            Pages::Edit(edit_state) => {
                render_edit_page(frame, main, edit_state)
            }
            Pages::Burst(burst_state) => {
                render_burst_page(frame, main, burst_state)
            }
        }
    }
}
