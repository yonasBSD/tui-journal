use std::{fs, io::BufWriter};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use super::*;

const STATE_FILE_NAME: &str = "state.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    pub sorter: Sorter,
    pub full_screen: bool,
}

impl AppState {
    pub fn load(settings: &Settings) -> anyhow::Result<Self> {
        // Move state file from legacy data directory to the new state directory
        // to avoid breaking changes on users.
        // TODO: Remove this after three releases.
        if settings.app_state_dir.is_none() {
            Self::move_legacy_state();
        }

        let state_path = Self::get_persist_path(settings)?;

        let state = if state_path.exists() {
            let state_file = File::open(state_path)
                .map_err(|err| anyhow!("Failed to load state file. Error info: {err}"))?;
            serde_json::from_reader(state_file)
                .map_err(|err| anyhow!("Failed to read state file. Error info: {err}"))?
        } else {
            AppState::default()
        };

        Ok(state)
    }

    fn get_persist_path(settings: &Settings) -> anyhow::Result<PathBuf> {
        if let Some(path) = settings.app_state_dir.as_ref() {
            Ok(path.join(STATE_FILE_NAME))
        } else {
            Self::default_persist_dir().map(|dir| dir.join(STATE_FILE_NAME))
        }
    }

    pub fn save(&self, settings: &Settings) -> anyhow::Result<()> {
        let state_path = Self::get_persist_path(settings)?;
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let state_file = File::create(state_path)?;
        let state_writer = BufWriter::new(state_file);

        serde_json::to_writer_pretty(state_writer, self)?;

        Ok(())
    }

    /// Return the default path of the directory used to persist the application state.
    /// It uses the state directories on supported platforms falling back to the data directory.
    pub fn default_persist_dir() -> anyhow::Result<PathBuf> {
        BaseDirs::new()
            .map(|base_dirs| {
                base_dirs
                    .state_dir()
                    .unwrap_or_else(|| base_dirs.data_dir())
                    .join("tui-journal")
            })
            .context("Config file path couldn't be retrieved")
    }

    /// Move app state from legacy path to the new one if the legacy exists and the new doesn't.
    fn move_legacy_state() {
        // Return early if operating system doesn't support `state_dir()`
        let state_path = match BaseDirs::new()
            .map(|base_dirs| base_dirs.state_dir().map(|state| state.join("tui-journal")))
        {
            Some(Some(state)) => state,
            _ => return,
        };

        // Gets legacy path which was used to store the state previously.
        let legacy_data_dir =
            match BaseDirs::new().map(|base_dirs| base_dirs.data_dir().join("tui-journal")) {
                Some(path) => path,
                None => return,
            };
        // Legacy already removed -> Done
        if !legacy_data_dir.exists() {
            return;
        }

        let legacy_state_file = legacy_data_dir.join(STATE_FILE_NAME);
        if !legacy_state_file.exists() {
            // Legacy dir exists but it has no files -> remove it -> Done.
            if let Err(err) = std::fs::remove_dir_all(&legacy_data_dir) {
                log::error!(
                    "Legacy State: Removing legacy directory failed. path: {}, Error {err}",
                    legacy_data_dir.display()
                );
            }
            return;
        }

        let new_state_file = state_path.join(STATE_FILE_NAME);
        if new_state_file.exists() {
            // New state file exists somehow -> Remove old state directory to avoid running this
            // again.
            if let Err(err) = std::fs::remove_dir_all(&legacy_data_dir) {
                log::error!(
                    "Legacy State: Removing legacy directory failed. path: {}, Error {err}",
                    legacy_data_dir.display()
                );
            }

            return;
        }

        if !state_path.exists() {
            // Create new state directory if not exists.
            if let Err(err) = std::fs::create_dir_all(&state_path) {
                log::error!(
                    "Legacy State: Creating state dir filed. Path: {}, Error {err}",
                    state_path.display()
                );
                return;
            }
        }

        // Move state file.
        if let Err(err) = std::fs::rename(legacy_state_file, new_state_file) {
            log::error!("Legacy State: Moving legacy state file failed. Error {err}");
            return;
        }

        // Finally remove legacy state directory.
        if let Err(err) = std::fs::remove_dir_all(&legacy_data_dir) {
            log::error!(
                "Legacy State: Removing legacy directory failed. path: {}, Error {err}",
                legacy_data_dir.display()
            );
        }
    }
}
