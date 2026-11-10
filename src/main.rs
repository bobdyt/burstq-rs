use color_eyre::eyre::WrapErr;

mod burst;
mod edit;
mod events;
mod model;
mod overview;
mod random;
mod storage;
mod term;
mod tui;

use term::init_terminal;
use term::install_panic_hook;
use term::restore;

use model::Model;

use events::run_event_loop;

fn main() -> color_eyre::Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal().wrap_err("Failed to init terminal")?;
    let mut model = Model::init();

    run_event_loop(&mut model, &mut terminal)
        .wrap_err("Fatal error in event loop")?;

    restore().wrap_err("Main function failed to restore terminal")?;
    Ok(())
}
