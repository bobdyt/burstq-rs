//use std::cell::RefCell;
//use std::rc::Rc;

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::{Row, Table, TableState};

use crate::model::{BurstItem, BurstPageState, WorkItem, WorkItemState};

pub fn render_burst_page(
    frame: &mut Frame,
    main: Rect,
    state: &BurstPageState,
) {
    let layout =
        Layout::vertical(vec![Constraint::Percentage(100)]).split(main);

    let table_area = layout[0];

    let mut table_state = TableState::default();
    table_state.select(state.table_idx);

    let header =
        Row::new(["ID", "Task"]).style(Style::new().bold()).bottom_margin(1);

    let items: Vec<Row> = state
        .bq
        .iter()
        .map(|bi: &BurstItem| {
            let wq = state.wq.borrow();
            let wi = get_work_item_by_wid(&wq, bi.wid.clone());
            return Row::new([wi.wid.clone(), wi.task.clone()]);
        })
        .collect();

    let widths = [Constraint::Percentage(10), Constraint::Percentage(80)];

    let table = Table::new(items, widths)
        .header(header)
        .column_spacing(1)
        .row_highlight_style(Style::new().on_black().bold())
        .column_highlight_style(Color::Gray)
        .cell_highlight_style(Style::new().reversed().yellow())
        .highlight_symbol(">")
        .block(Block::new().borders(Borders::ALL));

    frame.render_stateful_widget(table, table_area, &mut table_state);
}

pub fn get_work_item_by_wid(wq: &Vec<WorkItem>, wid: String) -> WorkItem {
    let matches_on_id =
        wq.iter().filter(|wi| wi.wid == wid).collect::<Vec<_>>();

    let item: WorkItem = match matches_on_id.first() {
        Some(wi) => (*wi).clone(),
        None => WorkItem {
            wid: "not-found-id".to_string(),
            state: WorkItemState::Pending,
            task: "FAILED TO FIND WORK ITEM".to_string(),
            details: "FAILED TO FIND WORK ITEM".to_string(),
            tags: "".to_string(),
        },
    };
    return item;
}
