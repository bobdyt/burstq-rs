use crossterm::event;
use crossterm::event::KeyCode;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use ratatui_textarea::TextArea;
use serde::{Deserialize, Serialize};

use crate::random::guid;
use crate::storage::load_work_queue;
use crate::storage::save_work_queue;

#[derive(Clone, Debug)]
pub enum BurstQueueState {
    Bursting,
    BurstEnded,
    BurstPending,
    Completed,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum WorkItemState {
    Edit,
    Pending,
    Burst,
    Completed,
    Forgotten,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WorkItem {
    // Work Item Identifier
    pub wid: String,

    // Work Item State
    pub state: WorkItemState,

    // One line description
    pub task: String,

    // Details
    pub details: String,

    // Tags
    pub tags: String,
}

impl WorkItem {
    fn loremipsum() -> WorkQueue {
        Rc::new(RefCell::new(vec![
            WorkItem {
                wid: "A0-3E-CC".to_string(),
                state: WorkItemState::Pending,
                task: "do it".to_string(),
                details: "".to_string(),
                tags: "demo,burst".to_string(),
            },
            WorkItem {
                wid: "3B-DD-1E".to_string(),
                state: WorkItemState::Pending,
                task: "plan that thing".to_string(),
                details: "".to_string(),
                tags: "demo".to_string(),
            },
        ]))
    }
}

#[derive(Clone, Debug)]
pub struct BurstItem {
    // Work item id
    pub wid: String,

    // Queue Order
    pub order: u32,

    // Queue state
    pub state: BurstQueueState,
}

pub type WorkQueue = Rc<RefCell<Vec<WorkItem>>>;
pub type BurstQueue = Vec<BurstItem>;

pub struct Model {
    // Project
    pub pr: String,

    // Burst queue
    pub bq: BurstQueue,

    // Work queue
    pub wq: WorkQueue,

    // UI
    pub ui: UIState,
}

impl Model {
    pub fn init() -> Model {
        let mut m = Model {
            pr: "Super Project".to_string(),
            bq: Vec::new(),
            wq: Rc::new(RefCell::new(Vec::new())),
            ui: UIState { terminate: false, history: Vec::new() },
        };
        m.ui.history.push(Pages::Overview(OverviewPageState {
            wq: m.wq.clone(),
            wq_table_idx: Some(0),
            show_details: false,
            preview_bq: true,
            preview_details: TextArea::new(vec!["".to_string()]),
            show_tag_nav: false,
            tag_nav_idx: 0,
        }));
        m.try_load();
        return m;
    }

    pub fn update_state(
        &mut self,
        events: Vec<StateChanges>,
    ) -> Option<Vec<StateChanges>> {
        let mut new_events = Vec::new();
        events.into_iter().for_each(|event| match event {
            StateChanges::GoBack => {
                self.ui.go_back();
            }
            StateChanges::Save => {
                self.save();
            }
            StateChanges::Terminate => {
                self.ui.terminate();
            }
            StateChanges::FlipWorkItemStateBurstOrPending(id) => {
                let mut wq = self.wq.borrow_mut();
                wq.iter_mut().for_each(|wi| {
                    if wi.wid == *id {
                        match wi.state {
                            WorkItemState::Pending => {
                                wi.state = WorkItemState::Burst
                            }
                            WorkItemState::Burst => {
                                wi.state = WorkItemState::Pending
                            }
                            _ => (),
                        }
                    }
                });
            }
            StateChanges::NewWorkItem(tags) => {
                let id = guid();
                let new_item = WorkItem {
                    wid: id.clone(),
                    state: WorkItemState::Pending,
                    task: "".to_string(),
                    details: "".to_string(),
                    tags: tags,
                };
                let mut wq = self.wq.borrow_mut();
                wq.push(new_item.clone());
                new_events.push(StateChanges::EditWorkItem(new_item));
            }
            StateChanges::EditWorkItem(wi) => {
                let item = self.get_work_item_by_wid(wi.wid.clone());
                self.ui.goto_page(Pages::Edit(EditPageState {
                    field: EditPageFields::Task,
                    mode: UserMode::Typing,
                    wid: wi.wid.clone(),
                    state: wi.state.clone(),
                    task: TextArea::new(
                        item.task.split("\n").map(String::from).collect(),
                    ),
                    details: TextArea::new(
                        item.details.split("\n").map(String::from).collect(),
                    ),
                    tags: TextArea::new(
                        item.tags.split("\n").map(String::from).collect(),
                    ),
                }));
            }
            StateChanges::UpdateWorkItem(new_wi) => {
                let mut wq = self.wq.borrow_mut();
                wq.iter_mut().for_each(|wi| {
                    if wi.wid == new_wi.wid {
                        wi.task = new_wi.task.clone();
                        wi.details = new_wi.details.clone();
                        wi.state = new_wi.state.clone();
                        wi.tags = new_wi.tags.clone();
                    }
                });
                new_events.push(StateChanges::Save);
            }
            StateChanges::DeleteWorkItem(wid) => {
                self.wq.borrow_mut().retain(|wi| wi.wid != wid);
                new_events.push(StateChanges::Save);
            }
            StateChanges::EnterBurstMode => {
                let wq = self.wq.borrow();
                let bq = wq
                    .clone()
                    .into_iter()
                    .filter(|wi| wi.state == WorkItemState::Burst)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|wi| BurstItem {
                        wid: wi.wid,
                        state: BurstQueueState::BurstPending,
                        order: 1,
                    })
                    .collect::<Vec<_>>();
                self.ui.goto_page(Pages::Burst(BurstPageState {
                    table_idx: Some(0),
                    bq: bq,
                    wq: self.wq.clone(),
                }));
            }
        });
        if new_events.len() == 0 {
            return None;
        } else {
            return Some(new_events);
        }
    }

    pub fn get_work_item_by_wid(&self, wid: String) -> WorkItem {
        let wq = self.wq.borrow();
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

    pub fn save(&self) -> color_eyre::Result<()> {
        save_work_queue(self.wq.borrow().to_vec())
    }

    pub fn try_load(&mut self) {
        if let Ok(wq) = load_work_queue() {
            let mut current_wq = self.wq.borrow_mut();
            *current_wq = wq;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StateChanges {
    FlipWorkItemStateBurstOrPending(String),
    NewWorkItem(String),
    EditWorkItem(WorkItem),
    UpdateWorkItem(WorkItem),
    DeleteWorkItem(String),
    Terminate,
    Save,
    GoBack,
    EnterBurstMode,
}

#[derive(Debug, Clone)]
pub enum Pages {
    Overview(OverviewPageState),
    Edit(EditPageState),
    Burst(BurstPageState),
}

pub struct UIState {
    pub history: Vec<Pages>,
    pub terminate: bool,
}

impl UIState {
    pub fn current_page(&mut self) -> Option<&mut Pages> {
        self.history.last_mut()
    }

    pub fn goto_page(&mut self, page: Pages) {
        self.history.push(page);
    }

    pub fn go_back(&mut self) {
        self.history.pop();
    }

    pub fn terminate(&mut self) {
        self.terminate = true;
    }

    pub fn handle_key(
        &mut self,
        event: event::KeyEvent,
    ) -> Option<Vec<StateChanges>> {
        if let Some(page) = self.current_page() {
            match page {
                Pages::Overview(state) => {
                    return state.handle_key(event);
                }
                Pages::Edit(state) => {
                    return state.handle_key(event);
                }
                Pages::Burst(state) => {
                    return state.handle_key(event);
                }
            }
        };
        None
    }
}

#[derive(Clone, Debug)]
pub struct OverviewPageState {
    pub wq_table_idx: Option<usize>,
    pub wq: WorkQueue,
    pub show_details: bool,
    pub preview_bq: bool,
    pub preview_details: TextArea<'static>,
    pub show_tag_nav: bool,
    pub tag_nav_idx: usize,
}

impl OverviewPageState {
    pub fn handle_key(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<Vec<StateChanges>> {
        match key.code {
            KeyCode::Char('j') => self.move_selection_down(),
            KeyCode::Char('k') => self.move_selection_up(),
            KeyCode::Char('x') => self.yield_flip_burst_or_pending(),
            KeyCode::Char('e') => self.yield_edit_item(),
            KeyCode::Char('n') => self.yield_new_item(),
            KeyCode::Char('d') => self.yield_delete_item(),
            KeyCode::Enter => Some(vec![StateChanges::EnterBurstMode]),
            KeyCode::Char('p') => self.toggle_show_details(),
            KeyCode::Char('b') => self.toggle_preview_bq(),
            KeyCode::Char('t') => self.toggle_tag_nav(),
            KeyCode::Char('h') => self.decr_tag_nav_idx(),
            KeyCode::Char('l') => self.incr_tag_nav_idx(),
            KeyCode::Esc => Some(vec![StateChanges::Terminate]),
            _ => None,
        }
    }

    fn toggle_tag_nav(&mut self) -> Option<Vec<StateChanges>> {
        self.show_tag_nav = !self.show_tag_nav;
        if self.show_tag_nav {
            self.tag_nav_idx = 0;
        }
        None
    }

    fn toggle_show_details(&mut self) -> Option<Vec<StateChanges>> {
        self.show_details = !self.show_details;
        None
    }

    fn toggle_preview_bq(&mut self) -> Option<Vec<StateChanges>> {
        self.preview_bq = !self.preview_bq;
        None
    }

    fn move_selection_down(&mut self) -> Option<Vec<StateChanges>> {
        if let Some(n) = self.wq_table_idx {
            let wq = self.wq.borrow();
            let n_items = wq.len();
            if n < n_items - 1 {
                self.wq_table_idx = Some(n + 1)
            }
        };
        None
    }

    fn move_selection_up(&mut self) -> Option<Vec<StateChanges>> {
        if let Some(n) = self.wq_table_idx {
            if n != 0 {
                self.wq_table_idx = Some(n - 1)
            }
        };
        None
    }

    fn reset_wq_selection(&mut self) -> Option<Vec<StateChanges>> {
        if let Some(n) = self.wq_table_idx {
            self.wq_table_idx = Some(0)
        };
        None
    }

    fn decr_tag_nav_idx(&mut self) -> Option<Vec<StateChanges>> {
        if self.tag_nav_idx != 0 {
            self.tag_nav_idx -= 1;
        } else {
            self.tag_nav_idx = self.get_tags().len() - 1;
        }
        self.reset_wq_selection()
    }

    fn incr_tag_nav_idx(&mut self) -> Option<Vec<StateChanges>> {
        if self.tag_nav_idx == self.get_tags().len() - 1 {
            self.tag_nav_idx = 0;
        } else {
            self.tag_nav_idx += 1;
        }
        self.reset_wq_selection()
    }

    pub fn get_tags(&self) -> Vec<String> {
        let mut tags: Vec<_> = self
            .wq
            .borrow()
            .clone()
            .into_iter()
            .filter(|wi| wi.tags != "".to_string())
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|wi| {
                wi.tags
                    .split(",")
                    .map(|tag| tag.trim().to_owned())
                    .collect::<Vec<String>>()
            })
            .collect::<HashSet<String>>()
            .into_iter()
            .collect::<Vec<_>>();
        tags.sort();
        return tags;
    }

    fn yield_flip_burst_or_pending(&self) -> Option<Vec<StateChanges>> {
        match self.get_selected_item() {
            Some(wi) => {
                return Some(vec![
                    StateChanges::FlipWorkItemStateBurstOrPending(
                        wi.wid.clone(),
                    ),
                ]);
            }
            None => return None,
        }
    }

    fn yield_edit_item(&self) -> Option<Vec<StateChanges>> {
        match self.get_selected_item() {
            Some(wi) => {
                return Some(vec![StateChanges::EditWorkItem(wi.clone())]);
            }
            None => return None,
        }
    }

    fn yield_new_item(&self) -> Option<Vec<StateChanges>> {
        let tags = self.get_tags();
        let mut tag = "".to_string();
        if tags.len() > 0 && self.show_tag_nav {
            tag = tags[self.tag_nav_idx].clone();
        }
        Some(vec![StateChanges::NewWorkItem(tag.to_string())])
    }

    fn yield_delete_item(&self) -> Option<Vec<StateChanges>> {
        match self.get_selected_item() {
            Some(wi) => {
                return Some(vec![StateChanges::DeleteWorkItem(
                    wi.wid.clone(),
                )]);
            }
            None => return None,
        }
    }

    fn get_selected_item(&self) -> Option<WorkItem> {
        if let Some(idx) = self.wq_table_idx {
            let wq = self.wq.borrow();
            if idx < wq.len() {
                let item = wq[idx].clone();
                return Some(item);
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UserMode {
    Typing,
    Navigating,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EditPageFields {
    Task,
    Details,
    Tags,
}

#[derive(Clone, Debug)]
pub struct EditPageState {
    pub field: EditPageFields,
    pub mode: UserMode,
    pub wid: String,
    pub state: WorkItemState,
    pub task: TextArea<'static>,
    pub details: TextArea<'static>,
    pub tags: TextArea<'static>,
}

impl EditPageState {
    pub fn handle_key(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<Vec<StateChanges>> {
        match self.mode {
            UserMode::Navigating => match key.code {
                KeyCode::Char('j') => {
                    match self.field {
                        EditPageFields::Tags => {
                            self.field = EditPageFields::Task
                        }
                        EditPageFields::Task => {
                            self.field = EditPageFields::Details
                        }
                        EditPageFields::Details => {
                            self.field = EditPageFields::Tags
                        }
                    };
                    None
                }
                KeyCode::Char('k') => {
                    match self.field {
                        EditPageFields::Tags => {
                            self.field = EditPageFields::Details
                        }
                        EditPageFields::Task => {
                            self.field = EditPageFields::Tags
                        }
                        EditPageFields::Details => {
                            self.field = EditPageFields::Task
                        }
                    };
                    None
                }
                KeyCode::Char('x') => None,
                KeyCode::Char('e') => {
                    self.mode = UserMode::Typing;
                    None
                }
                KeyCode::Char('n') => None,
                KeyCode::Esc => Some(vec![StateChanges::GoBack]),
                _ => None,
            },
            UserMode::Typing => match key.code {
                KeyCode::Esc => {
                    self.mode = UserMode::Navigating;
                    Some(vec![StateChanges::UpdateWorkItem(WorkItem {
                        wid: self.wid.clone(),
                        task: self.task.lines().join("\n"),
                        details: self.details.lines().join("\n"),
                        state: self.state.clone(),
                        tags: self.tags.lines().join("\n"),
                    })])
                }
                _ => match self.field {
                    EditPageFields::Tags => {
                        self.tags.input(key);
                        None
                    }
                    EditPageFields::Task => {
                        self.task.input(key);
                        None
                    }
                    EditPageFields::Details => {
                        self.details.input(key);
                        None
                    }
                },
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct BurstPageState {
    pub table_idx: Option<usize>,
    pub wq: WorkQueue,
    pub bq: BurstQueue,
}

impl BurstPageState {
    pub fn handle_key(
        &mut self,
        key: event::KeyEvent,
    ) -> Option<Vec<StateChanges>> {
        match key.code {
            KeyCode::Char('j') => self.move_selection_down(),
            KeyCode::Char('k') => self.move_selection_up(),
            //KeyCode::Char('x') => self.yield_flip_burst_or_pending(),
            //KeyCode::Char('e') => self.yield_edit_item(),
            //KeyCode::Char('n') => self.yield_new_item(),
            //KeyCode::Char('d') => self.yield_delete_item(),
            KeyCode::Esc => Some(vec![StateChanges::GoBack]),
            _ => None,
        }
    }

    fn move_selection_down(&mut self) -> Option<Vec<StateChanges>> {
        if let Some(n) = self.table_idx {
            let wq = self.wq.borrow();
            let n_items = wq.len();
            if n < n_items - 1 {
                self.table_idx = Some(n + 1)
            }
        };
        None
    }

    fn move_selection_up(&mut self) -> Option<Vec<StateChanges>> {
        if let Some(n) = self.table_idx {
            if n != 0 {
                self.table_idx = Some(n - 1)
            }
        };
        None
    }

    fn yield_flip_burst_or_pending(&self) -> Option<Vec<StateChanges>> {
        match self.get_selected_item() {
            Some(wi) => {
                return Some(vec![
                    StateChanges::FlipWorkItemStateBurstOrPending(
                        wi.wid.clone(),
                    ),
                ]);
            }
            None => return None,
        }
    }

    fn yield_edit_item(&self) -> Option<Vec<StateChanges>> {
        match self.get_selected_item() {
            Some(wi) => {
                return Some(vec![StateChanges::EditWorkItem(wi.clone())]);
            }
            None => return None,
        }
    }

    fn yield_new_item(&self) -> Option<Vec<StateChanges>> {
        Some(vec![StateChanges::NewWorkItem("".to_string())])
    }

    fn yield_delete_item(&self) -> Option<Vec<StateChanges>> {
        match self.get_selected_item() {
            Some(wi) => {
                return Some(vec![StateChanges::DeleteWorkItem(
                    wi.wid.clone(),
                )]);
            }
            None => return None,
        }
    }

    fn get_selected_item(&self) -> Option<WorkItem> {
        if let Some(idx) = self.table_idx {
            let wq = self.wq.borrow();
            if idx < wq.len() {
                let item = wq[idx].clone();
                return Some(item);
            }
        }
        None
    }
}
