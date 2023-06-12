use std::{path::PathBuf, str::FromStr};

use super::*;
use anyhow::anyhow;
use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Sqlite, SqlitePool,
};

pub struct SqliteDataProvide {
    pool: SqlitePool,
}

impl SqliteDataProvide {
    pub async fn from_file(file_path: PathBuf) -> anyhow::Result<Self> {
        let file_full_path = if file_path.exists() {
            tokio::fs::canonicalize(file_path).await?
        } else if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
            let parent_full_path = tokio::fs::canonicalize(parent).await?;
            parent_full_path.join(file_path.file_name().unwrap())
        } else {
            file_path
        };

        let db_url = format!("sqlite://{}", file_full_path.to_string_lossy());

        SqliteDataProvide::create(&db_url).await
    }

    pub async fn create(db_url: &str) -> anyhow::Result<Self> {
        if !Sqlite::database_exists(db_url).await? {
            log::trace!("Creating Database with the URL '{}'", db_url);
            Sqlite::create_database(db_url).await?;
        }

        // We are using the database as a normal file for one user.
        // Journal mode will causes problems with the synchronisation in our case and it must be
        // turned off
        let options = SqliteConnectOptions::from_str(db_url)?
            .journal_mode(SqliteJournalMode::Off)
            .synchronous(SqliteSynchronous::Off);

        let pool = SqlitePoolOptions::new().connect_with(options).await?;

        sqlx::migrate!("backend/src/sqlite/migrations")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DataProvider for SqliteDataProvide {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        todo!();

        // let entries = sqlx::query_as(
        //     r"SELECT * FROM entries
        // ORDER BY date DESC",
        // )
        // .fetch_all(&self.pool)
        // .await
        // .map_err(|err| {
        //     log::error!("Loading entries failed. Error Info {err}");
        //     anyhow!(err)
        // })?;
        //
        // Ok(entries)
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        todo!();
        // let entry = sqlx::query_as::<_, Entry>(
        //     r"INSERT INTO entries (title, date, content)
        //     VALUES($1, $2, $3)
        //     RETURNING *",
        // )
        // .bind(entry.title)
        // .bind(entry.date)
        // .bind(entry.content)
        // .fetch_one(&self.pool)
        // .await
        // .map_err(|err| {
        //     log::error!("Add entry field err: {}", err);
        //     anyhow!(err)
        // })?;
        //
        // Ok(entry)
    }

    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        sqlx::query(r"DELETE FROM entries WHERE id=$1")
            .bind(entry_id)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                log::error!("Delete entry failed. Error info: {err}");
                anyhow!(err)
            })?;

        Ok(())
    }

    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        todo!();
        // let entry = sqlx::query_as::<_, Entry>(
        //     r"UPDATE entries
        //     Set title = $1,
        //         date = $2,
        //         content = $3
        //     WHERE id = $4
        //     RETURNING *",
        // )
        // .bind(entry.title)
        // .bind(entry.date)
        // .bind(entry.content)
        // .bind(entry.id)
        // .fetch_one(&self.pool)
        // .await
        // .map_err(|err| {
        //     log::error!("Update entry failed. Error info {err}");
        //     anyhow!(err)
        // })?;
        //
        // Ok(entry)
    }

    async fn get_export_object(&self, entries_ids: &[u32]) -> anyhow::Result<EntriesDTO> {
        todo!();
        // let ids_text = entries_ids
        //     .iter()
        //     .map(|id| id.to_string())
        //     .collect::<Vec<String>>()
        //     .join(", ");
        //
        // let sql = format!(
        //     r"SELECT * FROM entries
        // WHERE id IN ({})
        // ORDER BY date DESC",
        //     ids_text
        // );
        //
        // let entries: Vec<Entry> = sqlx::query_as(sql.as_str())
        //     .fetch_all(&self.pool)
        //     .await
        //     .map_err(|err| {
        //         log::error!("Loading entries failed. Error Info {err}");
        //         anyhow!(err)
        //     })?;
        //
        // let entry_drafts = entries.into_iter().map(EntryDraft::from_entry).collect();
        //
        // Ok(EntriesDTO::new(entry_drafts))
    }

    async fn import_entries(&self, entries_dto: EntriesDTO) -> anyhow::Result<()> {
        debug_assert_eq!(
            TRANSFER_DATA_VERSION, entries_dto.version,
            "Version mismatches check if there is a need to do a converting to the data"
        );

        for entry_darft in entries_dto.entries {
            self.add_entry(entry_darft).await?;
        }

        Ok(())
    }
}
