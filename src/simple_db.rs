use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub completed: bool,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub priority: Option<String>,
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
        sqlx::query("CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY, username TEXT UNIQUE, email TEXT UNIQUE, password_hash TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, updated_at DATETIME DEFAULT CURRENT_TIMESTAMP)")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS todos (id TEXT PRIMARY KEY, text TEXT, completed BOOLEAN DEFAULT FALSE, category TEXT, tags TEXT, priority TEXT, due_date DATETIME, user_id TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, updated_at DATETIME DEFAULT CURRENT_TIMESTAMP)")
            .execute(&pool)
            .await?;

        Ok(Database { pool })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn create_todo(&self, new_todo: NewTodo, user_id: Option<&str>) -> Result<Todo, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let tags_json = new_todo
            .tags
            .map(|tags| serde_json::to_string(&tags).unwrap_or_default());

        sqlx::query("INSERT INTO todos (id, text, completed, category, tags, priority, due_date, user_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&id)
            .bind(&new_todo.text)
            .bind(false)
            .bind(&new_todo.category)
            .bind(&tags_json)
            .bind(&new_todo.priority)
            .bind(&new_todo.due_date)
            .bind(user_id)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await?;

        Ok(Todo {
            id,
            text: new_todo.text,
            completed: false,
            category: new_todo.category,
            tags: tags_json,
            priority: new_todo.priority,
            due_date: new_todo.due_date,
            user_id: user_id.map(String::from),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_todos(&self, user_id: Option<&str>) -> Result<Vec<Todo>, sqlx::Error> {
        let rows = match user_id {
            Some(uid) => {
                sqlx::query("SELECT id, text, completed, category, tags, priority, due_date, user_id, created_at, updated_at FROM todos WHERE user_id = ? OR user_id IS NULL ORDER BY created_at DESC")
                    .bind(uid)
                    .fetch_all(&self.pool)
                    .await?
            }
            None => {
                sqlx::query("SELECT id, text, completed, category, tags, priority, due_date, user_id, created_at, updated_at FROM todos WHERE user_id IS NULL ORDER BY created_at DESC")
                    .fetch_all(&self.pool)
                    .await?
            }
        };

        let mut todos = Vec::new();
        for row in rows {
            todos.push(Todo {
                id: row.get("id"),
                text: row.get("text"),
                completed: row.get("completed"),
                category: row.get("category"),
                tags: row.get("tags"),
                priority: row.get("priority"),
                due_date: row.get("due_date"),
                user_id: row.get("user_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(todos)
    }

    pub async fn toggle_todo(&self, id: &str) -> Result<Option<Todo>, sqlx::Error> {
        let now = Utc::now();

        sqlx::query("UPDATE todos SET completed = NOT completed, updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        let row = sqlx::query("SELECT id, text, completed, category, tags, priority, due_date, user_id, created_at, updated_at FROM todos WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Todo {
                id: row.get("id"),
                text: row.get("text"),
                completed: row.get("completed"),
                category: row.get("category"),
                tags: row.get("tags"),
                priority: row.get("priority"),
                due_date: row.get("due_date"),
                user_id: row.get("user_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_categories(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query("SELECT DISTINCT category FROM todos WHERE category IS NOT NULL")
            .fetch_all(&self.pool)
            .await?;

        Ok(
            rows.into_iter()
                .filter_map(|row| row.get::<Option<String>, _>("category"))
                .collect(),
        )
    }
}
