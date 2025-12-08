use anyhow::Result;
use sqlx::SqlitePool;

use crate::db::model::ImapFolderState;

pub async fn load_or_create_imap_folder_state(
    db: &SqlitePool,
    folder_id: i64,
) -> Result<ImapFolderState> {
    if let Some(state) = sqlx::query_as!(
        ImapFolderState,
        r#"
        SELECT folder_id, uid_validity, highest_modseq, uid_next, highest_uid, last_sync_ts
        FROM imap_folder_state WHERE folder_id = ?
        "#,
        folder_id
    )
    .fetch_optional(db)
    .await?
    {
        return Ok(state);
    }

    sqlx::query!(
        r#"
        INSERT INTO imap_folder_state (folder_id, uid_validity, highest_modseq, uid_next, highest_uid, last_sync_ts)
        VALUES (?, NULL, NULL, NULL, NULL, NULL)
        "#,
        folder_id
    )
    .execute(db)
    .await?;

    Ok(ImapFolderState {
        folder_id,
        uid_validity: None,
        highest_modseq: None,
        uid_next: None,
        highest_uid: None,
        last_sync_ts: None,
    })
}
