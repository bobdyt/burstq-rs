use std::time::Duration;

use crate::model::Model;
use crate::term::Terminal;
use crate::tui::main_view;

use crossterm::event;
use crossterm::event::Event;
//use crossterm::event::KeyCode;

use color_eyre::eyre::WrapErr;

pub fn run_event_loop(
    model: &mut Model,
    terminal: &mut Terminal,
) -> color_eyre::Result<()> {
    while !model.ui.terminate {
        terminal
            .draw(|f| main_view(&mut model.ui, f))
            .wrap_err("Event loop failed to draw main view")?;

        let key_event = poll().wrap_err("Event loop failed to handle event")?;

        if let Some(key) = key_event {
            let mut next_change_batch = model.ui.handle_key(key);
            while let Some(ref batch) = next_change_batch {
                if batch.len() > 0 {
                    next_change_batch = model.update_state(batch.clone());
                } else {
                }
            }
        }
    }
    Ok(())
}

fn poll() -> color_eyre::Result<Option<event::KeyEvent>> {
    if event::poll(Duration::from_millis(250))?
        && let Event::Key(key) = event::read()?
        && key.kind == event::KeyEventKind::Press
    {
        return Ok(Some(key));
    }
    Ok(None)
}
