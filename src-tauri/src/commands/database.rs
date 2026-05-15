//! M6 — Database commands: auto-create testing DB, dump, restore, open CLI

use crate::core::error::AppError;

/// Auto-create `{project_id}_testing` DB in global PostgreSQL container.
#[tauri::command]
pub async fn create_testing_db(_project_id: String) -> Result<(), AppError> {
    todo!("implement in 04-04-PLAN")
}

/// Dump project database via pg_dump to a user-chosen file path.
#[tauri::command]
pub async fn dump_db(_project_id: String, _dest_path: String) -> Result<(), AppError> {
    todo!("implement in 04-04-PLAN")
}

/// Restore project database from a user-chosen file path.
#[tauri::command]
pub async fn restore_db(_project_id: String, _src_path: String) -> Result<(), AppError> {
    todo!("implement in 04-04-PLAN")
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "stub — implement in 04-04-PLAN"]
    fn auto_create_testing_db_runs_correct_sql() {}
}
