//use std::cell::RefCell;
//use std::rc::Rc;

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Padding;
use ratatui::widgets::{Row, Table, TableState};

use crate::model::{BurstItem, BurstPageState, WorkItem, WorkItemState};

pub fn render_burst_page(
    frame: &mut Frame,
    main: Rect,
    state: &mut BurstPageState,
) {
    let mut table_state = TableState::default();
    table_state.select(state.table_idx);

    let header =
        Row::new(["ID", "Task"]).style(Style::new().bold()).bottom_margin(1);

    let wq = state.wq.borrow();
    let items: Vec<Row> = state
        .bq
        .iter()
        .map(|bi: &BurstItem| {
            let wi = get_work_item_by_wid(&wq, bi.wid.clone());
            let lines =
                wi.brief.split("\n").map(String::from).collect::<Vec<String>>();
            let head = lines[0].clone();
            return Row::new([wi.wid.clone(), head]);
        })
        .collect();

    let mut constraints = Vec::new();
    constraints.push(Constraint::Percentage(50));
    constraints.push(Constraint::Fill(1));

    let direction = if main.width >= 80 {
        Direction::Horizontal
    } else {
        Direction::Vertical
    };

    let layout = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(main);

    let table_area = layout[0];
    let wi_area = layout[1];

    let widths = [Constraint::Percentage(10), Constraint::Percentage(80)];
    let table = Table::new(items, widths)
        .header(header)
        .column_spacing(1)
        .row_highlight_style(Style::new().on_black().bold())
        .column_highlight_style(Color::Gray)
        .cell_highlight_style(Style::new().reversed().yellow())
        .block(Block::new().borders(Borders::ALL).title(" Burst queue "));

    frame.render_stateful_widget(table, table_area, &mut table_state);

    state.brief.set_block(
        Block::bordered().title(" Brief ").padding(Padding::horizontal(1)),
    );
    frame.render_widget(&state.brief, wi_area);
}

pub fn get_work_item_by_wid(wq: &Vec<WorkItem>, wid: String) -> WorkItem {
    let matches_on_id =
        wq.iter().filter(|wi| wi.wid == wid).collect::<Vec<_>>();

    let item: WorkItem = match matches_on_id.first() {
        Some(wi) => (*wi).clone(),
        None => WorkItem {
            wid: "not-found-id".to_string(),
            state: WorkItemState::Pending,
            brief: "FAILED TO FIND WORK ITEM".to_string(),
            tags: "".to_string(),
        },
    };
    return item;
}
