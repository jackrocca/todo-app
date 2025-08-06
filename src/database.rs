use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Row};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub completed: bool,
    pub category: Option<String>,
    pub tags: Option<String>, // JSON string of tags array
    pub priority: Option<String>, // "high", "medium", "low"
    pub due_date: Option<DateTime<Utc>>,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewTodo {
    pub text: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub priority: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        
        // Create tables if they don't exist
        sqlx::query("CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY NOT NULL, username TEXT UNIQUE NOT NULL, email TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE TABLE IF NOT EXISTS todos (id TEXT PRIMARY KEY NOT NULL, text TEXT NOT NULL, completed BOOLEAN NOT NULL DEFAULT FALSE, category TEXT, tags TEXT, priority TEXT CHECK (priority IN ('high', 'medium', 'low')), due_date DATETIME, user_id TEXT REFERENCES users(id), created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP)")
            .execute(&pool).await?;
        
        Ok(Database { pool })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn create_todo(&self, new_todo: NewTodo, user_id: Option<&str>) -> Result<Todo, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let tags_json = new_todo.tags.map(|tags| serde_json::to_string(&tags).unwrap_or_default());

        let todo = sqlx::query_as(
            Todo,
            r#"
            INSERT INTO todos (id, text, completed, category, tags, priority, due_date, user_id, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            RETURNING id, text, completed, category, tags, priority, due_date as "due_date: DateTime<Utc>", user_id, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
            "#,
            id, new_todo.text, false, new_todo.category, tags_json, new_todo.priority, new_todo.due_date, user_id, now, now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(todo)
    }

    pub async fn get_todos(&self, user_id: Option<&str>) -> Result<Vec<Todo>, sqlx::Error> {
        let todos = match user_id {
            Some(uid) => {
                sqlx::query_as(
                    Todo,
                    r#"
                    SELECT id, text, completed, category, tags, priority, due_date as "due_date: DateTime<Utc>", user_id, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
                    FROM todos
                    WHERE user_id = ?1 OR user_id IS NULL
                    ORDER BY created_at DESC
                    "#,
                    uid
                )
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    Todo,
                    r#"
                    SELECT id, text, completed, category, tags, priority, due_date as "due_date: DateTime<Utc>", user_id, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
                    FROM todos
                    WHERE user_id IS NULL
                    ORDER BY created_at DESC
                    "#
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(todos)
    }

    pub async fn toggle_todo(&self, id: &str) -> Result<Option<Todo>, sqlx::Error> {
        let now = Utc::now();
        
        let todo = sqlx::query_as(
            Todo,
            r#"
            UPDATE todos
            SET completed = NOT completed, updated_at = ?2
            WHERE id = ?1
            RETURNING id, text, completed, category, tags, priority, due_date as "due_date: DateTime<Utc>", user_id, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
            "#,
            id, now
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(todo)
    }

    pub async fn delete_todo(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM todos WHERE id = ?1", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_todo(&self, id: &str, new_todo: NewTodo) -> Result<Option<Todo>, sqlx::Error> {
        let now = Utc::now();
        let tags_json = new_todo.tags.map(|tags| serde_json::to_string(&tags).unwrap_or_default());

        let todo = sqlx::query_as(
            Todo,
            r#"
            UPDATE todos
            SET text = ?2, category = ?3, tags = ?4, priority = ?5, due_date = ?6, updated_at = ?7
            WHERE id = ?1
            RETURNING id, text, completed, category, tags, priority, due_date as "due_date: DateTime<Utc>", user_id, created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
            "#,
            id, new_todo.text, new_todo.category, tags_json, new_todo.priority, new_todo.due_date, now
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(todo)
    }

    pub async fn get_categories(&self) -> Result<Vec<String>, sqlx::Error> {
        let categories = sqlx::query("SELECT DISTINCT category FROM todos WHERE category IS NOT NULL")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .filter_map(|row| row.category)
            .collect();

        Ok(categories)
    }
}