use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;
use ratatui::widgets::{Row, Table, TableState};
use ratatui_textarea::TextArea;
use std::collections::HashSet;

use crate::model::{OverviewPageState, WorkItem, WorkItemState};

pub fn render_overview(
    frame: &mut Frame,
    main: Rect,
    state: &mut OverviewPageState,
) {
    let mut wq_rows: Vec<Row> = Vec::new();
    let mut bq_rows = Vec::new();
    let tags = state.get_tags();

    state
        .wq
        .borrow()
        .iter()
        .filter(|wi| {
            let mut selected_tag = "".to_string();
            if state.show_tag_nav {
                selected_tag = tags[state.tag_nav_idx].to_string();
            }
            return wi.tags.contains(&selected_tag);
        })
        .for_each(|wi| {
            let burst_symbol = match wi.state {
                WorkItemState::Burst => "x".to_string(),
                _ => "".to_string(),
            };

            wq_rows.push(Row::new([
                wi.wid.clone(),
                wi.task.clone(),
                wi.tags.clone(),
                burst_symbol.clone(),
            ]));
            if burst_symbol == "x".to_string() {
                bq_rows.push(Row::new([
                    wi.wid.clone(),
                    wi.task.clone(),
                    wi.tags.clone(),
                ]))
            }
        });

    let mut verticals = Vec::new();
    verticals.push(Constraint::Percentage(if state.preview_bq {
        20
    } else {
        0
    }));
    verticals.push(Constraint::Percentage(if state.show_details {
        30
    } else {
        0
    }));
    verticals.push(Constraint::Length(if state.show_tag_nav { 3 } else { 0 }));
    verticals.push(Constraint::Fill(1));
    let vlayout = Layout::vertical(verticals).split(main);

    if state.show_details {
        let wq = state.wq.borrow();
        if let Some(idx) = state.wq_table_idx {
            let wi = wq[idx].clone();
            let mut details = TextArea::from(
                vec![
                    wi.task.lines().map(String::from).collect::<Vec<_>>(),
                    vec!["\n".to_string()],
                    wi.details.lines().map(String::from).collect::<Vec<_>>(),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            );
            details.set_block(
                Block::bordered()
                    .title(" Preview ")
                    .padding(Padding::uniform(1)),
            );
            details.set_cursor_line_style(Style::default());
            details.set_cursor_style(Style::default());
            frame.render_widget(&details, vlayout[1]);
        }
    }

    if state.show_tag_nav {
        let mut spans = Vec::new();
        for (i, tag) in tags.into_iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(format!(" ")));
            }
            let span = Span::raw(format!("{}", tag));
            if i == state.tag_nav_idx {
                spans.push(span.bold());
            } else {
                spans.push(span)
            }
        }
        let line = Line::from(spans);
        let mut paragraph = Paragraph::new(line).block(
            Block::bordered().title(" Tags ").padding(Padding::horizontal(1)),
        );
        frame.render_widget(&paragraph, vlayout[2]);
    }

    if state.preview_bq {
        let mut bq_table_state = TableState::default();

        let bq_header = Row::new(["ID", "Task", "Tags"])
            .style(Style::new().bold())
            .bottom_margin(1);

        let bq_widths = [
            Constraint::Percentage(10),
            Constraint::Percentage(30),
            Constraint::Percentage(10),
        ];

        let bq_table = Table::new(bq_rows, bq_widths)
            .header(bq_header)
            .column_spacing(1)
            .row_highlight_style(Style::new().on_black().bold())
            .column_highlight_style(Color::Gray)
            .cell_highlight_style(Style::new().reversed().yellow())
            .highlight_symbol(">")
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title(" Burst queue ")
                    .padding(Padding::horizontal(1)),
            );

        frame.render_stateful_widget(bq_table, vlayout[0], &mut bq_table_state);
    }

    let mut wq_table_state = TableState::default();
    wq_table_state.select(state.wq_table_idx);

    let wq_header = Row::new(["ID", "Task", "Tags", "Burst"])
        .style(Style::new().bold())
        .bottom_margin(1);

    let wq_widths = [
        Constraint::Percentage(10),
        Constraint::Percentage(30),
        Constraint::Percentage(50),
        Constraint::Percentage(10),
    ];

    let wq_table = Table::new(wq_rows, wq_widths)
        .header(wq_header)
        .column_spacing(1)
        .row_highlight_style(Style::new().on_black().bold())
        .column_highlight_style(Color::Gray)
        .cell_highlight_style(Style::new().reversed().yellow())
        .highlight_symbol(">")
        .block(
            Block::new()
                .borders(Borders::ALL)
                .title(" Work queue ")
                .padding(Padding::horizontal(1)),
        );

    frame.render_stateful_widget(wq_table, vlayout[3], &mut wq_table_state);
}
