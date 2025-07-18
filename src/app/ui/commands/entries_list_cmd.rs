use std::{collections::HashMap, env};

use crate::app::{App, UIComponents, external_editor, ui::*};

use backend::DataProvider;

use scopeguard::defer;

use super::{
    CmdResult,
    editor_cmd::{discard_current_content, exec_save_entry_content},
};

pub fn exec_select_prev_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::SelectedPrevEntry));
    } else {
        select_prev_entry(1, ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

fn select_prev_entry<D: DataProvider>(
    step: usize,
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) {
    let prev_id = ui_components
        .entries_list
        .state
        .selected()
        .map(|index| index.saturating_sub(step))
        .and_then(|prev_index| {
            app.get_active_entries()
                .nth(prev_index)
                .map(|entry| entry.id)
        });

    if prev_id.is_some() {
        ui_components.set_current_entry(prev_id, app);
    }
}

pub async fn continue_select_prev_entry<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            select_prev_entry(1, ui_components, app);
        }
        MsgBoxResult::No => select_prev_entry(1, ui_components, app),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_select_next_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::SelectedNextEntry));
    } else {
        select_next_entry(1, ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

fn select_next_entry<D: DataProvider>(
    step: usize,
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) {
    let next_id = ui_components
        .entries_list
        .state
        .selected()
        .and_then(|index| index.checked_add(step))
        .and_then(|next_index| {
            app.get_active_entries()
                .nth(next_index)
                .or_else(|| app.get_active_entries().next_back())
                .map(|entry| entry.id)
        });

    if next_id.is_some() {
        ui_components.set_current_entry(next_id, app);
    }
}

pub async fn continue_select_next_entry<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            select_next_entry(1, ui_components, app);
        }
        MsgBoxResult::No => select_next_entry(1, ui_components, app),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_create_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::CreateEntry));
    } else {
        create_entry(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn create_entry<D: DataProvider>(ui_components: &mut UIComponents, app: &App<D>) {
    ui_components
        .popup_stack
        .push(Popup::Entry(Box::new(EntryPopup::new_entry(&app.settings))));
}

pub async fn continue_create_entry<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            create_entry(ui_components, app);
        }
        MsgBoxResult::No => create_entry(ui_components, app),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_edit_current_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::EditCurrentEntry));
    } else {
        edit_current_entry(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

fn edit_current_entry<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    if let Some(entry) = app.get_current_entry() {
        ui_components
            .popup_stack
            .push(Popup::Entry(Box::new(EntryPopup::from_entry(entry))));
    }
}

pub async fn continue_edit_current_entry<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            edit_current_entry(ui_components, app);
        }
        MsgBoxResult::No => {
            discard_current_content(ui_components, app);
            edit_current_entry(ui_components, app);
        }
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_delete_current_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &App<D>,
) -> CmdResult {
    if app.current_entry_id.is_some() {
        let msg = MsgBoxType::Question("Do you want to remove the current journal?".into());
        let msg_actions = MsgBoxActions::YesNo;
        ui_components.show_msg_box(msg, msg_actions, Some(UICommand::DeleteCurrentEntry));
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn continue_delete_current_entry<D: DataProvider>(
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Yes => {
            app.delete_entry(
                app.current_entry_id
                    .expect("current entry must have a value"),
            )
            .await?;
        }
        MsgBoxResult::No => {}
        _ => unreachable!(
            "{:?} not implemented for delete current entry",
            msg_box_result
        ),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_export_entry_content<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::ExportEntryContent));
    } else {
        export_entry_content(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn export_entry_content<D: DataProvider>(ui_components: &mut UIComponents, app: &App<D>) {
    if let Some(entry) = app.get_current_entry() {
        match ExportPopup::create_entry_content(entry, app) {
            Ok(popup) => ui_components
                .popup_stack
                .push(Popup::Export(Box::new(popup))),
            Err(err) => ui_components
                .show_err_msg(format!("Error while creating export dialog.\n Err: {err}")),
        }
    }
}

pub async fn continue_export_entry_content<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            export_entry_content(ui_components, app);
        }
        MsgBoxResult::No => {
            discard_current_content(ui_components, app);
            export_entry_content(ui_components, app);
        }
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn exec_edit_in_external_editor<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::EditInExternalEditor));
    } else {
        edit_in_external_editor(ui_components, app).await?;
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn edit_in_external_editor<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
) -> anyhow::Result<()> {
    use tokio::fs;

    if let Some(entry) = app.get_current_entry() {
        const TEMP_FILENAME: &str = "tui_journal";
        let temp_extension = &app.settings.external_editor.temp_file_extension;

        let file_name = if !temp_extension.is_empty() {
            format!("{TEMP_FILENAME}.{temp_extension}")
        } else {
            String::from(TEMP_FILENAME)
        };

        let file_path = env::temp_dir().join(file_name);

        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }

        fs::write(&file_path, entry.content.as_str()).await?;

        defer! {
        std::fs::remove_file(&file_path).expect("Temp File couldn't be deleted");
        }

        app.redraw_after_restore = true;

        external_editor::open_editor(&file_path, &app.settings).await?;

        if file_path.exists() {
            let new_content = fs::read_to_string(&file_path).await?;
            ui_components.editor.set_entry_content(&new_content, app);
            ui_components.change_active_control(ControlType::EntriesList);

            if app.settings.external_editor.auto_save {
                exec_save_entry_content(ui_components, app).await?;
            }
        }
    }

    Ok(())
}

pub async fn continue_edit_in_external_editor<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            edit_in_external_editor(ui_components, app).await?;
        }
        MsgBoxResult::No => edit_in_external_editor(ui_components, app).await?,
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_show_filter<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::ShowFilter));
    } else {
        show_filter(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

fn show_filter<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let tags = app.get_all_tags();
    ui_components
        .popup_stack
        .push(Popup::Filter(Box::new(FilterPopup::new(
            tags,
            app.filter.clone(),
        ))));
}

pub async fn continue_show_filter<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            show_filter(ui_components, app);
        }
        MsgBoxResult::No => {
            discard_current_content(ui_components, app);
            show_filter(ui_components, app);
        }
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_reset_filter<D: DataProvider>(app: &mut App<D>) -> CmdResult {
    app.apply_filter(None);

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_cycle_tag_filter<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::CycleTagFilter));
    } else {
        app.cycle_tags_in_filter();
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn continue_cycle_tag_filter<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            app.cycle_tags_in_filter();
        }
        MsgBoxResult::No => {
            discard_current_content(ui_components, app);
            app.cycle_tags_in_filter();
        }
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_show_fuzzy_find<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::ShowFuzzyFind));
    } else {
        show_fuzzy_find(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

fn show_fuzzy_find<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let entries: HashMap<u32, String> = app
        .get_active_entries()
        .map(|entry| (entry.id, entry.title.to_owned()))
        .collect();
    ui_components
        .popup_stack
        .push(Popup::FuzzFind(Box::new(FuzzFindPopup::new(entries))));
}

pub async fn continue_fuzzy_find<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            show_fuzzy_find(ui_components, app);
        }
        MsgBoxResult::No => {
            discard_current_content(ui_components, app);
            show_fuzzy_find(ui_components, app);
        }
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_toggle_full_screen_mode<D: DataProvider>(app: &mut App<D>) -> CmdResult {
    app.state.full_screen = !app.state.full_screen;
    Ok(HandleInputReturnType::Handled)
}

pub fn exec_show_sort_options<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::ShowSortOptions));
    } else {
        show_sort_options(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

fn show_sort_options<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    ui_components
        .popup_stack
        .push(Popup::Sort(Box::new(SortPopup::new(&app.state.sorter))));
}

pub async fn continue_show_sort_options<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            show_sort_options(ui_components, app);
        }
        MsgBoxResult::No => {
            // Discard the current content explicitly because it doesn't get discarded if the sort
            // was cancelled which could confuse the users
            discard_current_content(ui_components, app);

            show_sort_options(ui_components, app);
        }
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn go_to_top_entry<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let top_id = app.get_active_entries().next().map(|entry| entry.id);

    if top_id.is_some() {
        ui_components.set_current_entry(top_id, app);
    }
}

pub fn go_to_bottom_entry<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let top_id = app.get_active_entries().next_back().map(|entry| entry.id);

    if top_id.is_some() {
        ui_components.set_current_entry(top_id, app);
    }
}

pub fn page_up_entries<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let step = app.settings.get_scroll_per_page();

    select_prev_entry(step, ui_components, app);
}

pub fn page_down_entries<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let step = app.settings.get_scroll_per_page();

    select_next_entry(step, ui_components, app);
}
