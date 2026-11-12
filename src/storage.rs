use directories::ProjectDirs;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::WrapErr;

use crate::model::WorkItem;

const WORK_QUEUE: &str = "wq.json";

fn get_path(filename: &str) -> color_eyre::Result<PathBuf> {
    let dirs = ProjectDirs::from("tool", "burstq", "cli")
        .wrap_err("Could not determine config directory")?;

    let dir = dirs.data_dir();
    fs::create_dir_all(dir).wrap_err("Failed to create data directory")?;
    Ok(dir.join(filename))
}

pub fn save_work_queue(wq: Vec<WorkItem>) -> color_eyre::Result<()> {
    let path =
        get_path(WORK_QUEUE).wrap_err("Failed to get path to work queue")?;
    let serialized_wq = serde_json::to_string_pretty(&wq)
        .wrap_err("Failed to serialize work items to string")?;
    fs::write(&path, serialized_wq).wrap_err("Failed to store work items")
}

pub fn load_work_queue() -> color_eyre::Result<Vec<WorkItem>> {
    let path =
        get_path(WORK_QUEUE).wrap_err("Failed to get path to work queue")?;
    let file = File::open(path)?;
    let wq = serde_json::from_reader(file)
        .wrap_err("Failed to read workqueue file")?;
    return Ok(wq);
}
