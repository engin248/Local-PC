use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub user_request: String,
    pub status: String,
    pub planning_status: String,
    pub execution_status: String,
    pub current_gate: Option<String>,
    pub last_valid_state_id: Option<String>,
    pub risk_level: String,
    pub approval_status: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskIntakeRequest {
    pub title: String,
    pub user_request: String,
}

pub fn create_task(req: TaskIntakeRequest) -> Result<Task, String> {
    TaskIntake::intake_task(&req.title, &req.user_request)
}

pub struct TaskIntake;

impl TaskIntake {
    pub fn intake_task(title: &str, request: &str) -> Result<Task, String> {
        let task = Task {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            user_request: request.to_string(),
            status: "pending".to_string(),
            planning_status: "planning_incomplete".to_string(),
            execution_status: "not_started".to_string(),
            current_gate: None,
            last_valid_state_id: None,
            risk_level: "low".to_string(),
            approval_status: "pending_approval".to_string(),
            created_at: None,
        };

        // Write to DB
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, current_gate, last_valid_state_id, risk_level, approval_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                task.id,
                task.title,
                task.user_request,
                task.status,
                task.planning_status,
                task.execution_status,
                task.current_gate,
                task.last_valid_state_id,
                task.risk_level,
                task.approval_status
            ],
        ).map_err(|e| e.to_string())?;

        Ok(task)
    }

    pub fn get_task(id: &str) -> Result<Task, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, title, user_request, status, planning_status, execution_status, current_gate, last_valid_state_id, risk_level, approval_status, created_at FROM tasks WHERE id = ?1")
            .map_err(|e| e.to_string())?;

        let task = stmt
            .query_row(params![id], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    user_request: row.get(2)?,
                    status: row.get(3)?,
                    planning_status: row.get(4)?,
                    execution_status: row.get(5)?,
                    current_gate: row.get(6)?,
                    last_valid_state_id: row.get(7)?,
                    risk_level: row.get(8)?,
                    approval_status: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?;

        Ok(task)
    }
}
