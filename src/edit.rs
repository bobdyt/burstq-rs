use crate::model::EditPageFields;
use crate::model::EditPageState;
use crate::model::UserMode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Padding;
use ratatui_textarea::TextArea;

pub fn render_edit_page(
    frame: &mut Frame,
    main: Rect,
    state: &mut EditPageState,
) {
    let layout = Layout::vertical(vec![
        Constraint::Length(3),
        Constraint::Percentage(30),
        Constraint::Fill(1),
    ])
    .split(main);

    let tags_area = layout[0];
    state.tags.set_block(
        Block::bordered()
            .title(" Tags ")
            .padding(Padding::horizontal(1))
            .border_style(get_border_style_on_selected(
                EditPageFields::Tags,
                &state.field,
                &state.mode,
            )),
    );
    set_read_only_style(
        &mut state.tags,
        EditPageFields::Tags,
        &state.field,
        &state.mode,
    );
    frame.render_widget(&state.tags, tags_area);

    let task_area = layout[1];
    state.task.set_block(
        Block::bordered()
            .title(" Task ")
            .padding(Padding::uniform(1))
            .border_style(get_border_style_on_selected(
                EditPageFields::Task,
                &state.field,
                &state.mode,
            )),
    );
    set_read_only_style(
        &mut state.task,
        EditPageFields::Task,
        &state.field,
        &state.mode,
    );
    frame.render_widget(&state.task, task_area);

    let details_area = layout[2];
    state.details.set_block(
        Block::bordered()
            .title(" Details ")
            .padding(Padding::uniform(1))
            .border_style(get_border_style_on_selected(
                EditPageFields::Details,
                &state.field,
                &state.mode,
            )),
    );
    set_read_only_style(
        &mut state.details,
        EditPageFields::Details,
        &state.field,
        &state.mode,
    );
    frame.render_widget(&state.details, details_area);
}

fn set_read_only_style(
    widget: &mut TextArea<'static>,
    target: EditPageFields,
    selected: &EditPageFields,
    mode: &UserMode,
) {
    if target != *selected || *mode == UserMode::Navigating {
        widget.set_cursor_line_style(Style::default());
        widget.set_cursor_style(Style::default());
    }
}

fn get_border_style_on_selected(
    target: EditPageFields,
    selected: &EditPageFields,
    mode: &UserMode,
) -> Style {
    if target == *selected {
        match mode {
            UserMode::Navigating => Style::default().fg(Color::Green),
            UserMode::Typing => Style::default().fg(Color::Red),
        }
    } else {
        Style::default()
    }
}
