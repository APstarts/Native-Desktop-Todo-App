use crate::models::todo::Todo;
use sqlx::{Error, SqlitePool};

// The repository owns clonable sqlx pool
#[derive(Clone)]
pub struct TodoRepository {
    pool: SqlitePool,
}

impl TodoRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl TodoRepository {
    pub async fn get_all(&self) -> Result<Vec<Todo>, sqlx::Error> {
        let rows = sqlx::query!(r#"SELECT id, title, COMPLETED FROM todos ORDER BY id"#)
            .fetch_all(&self.pool)
            .await?;

        let todos = rows
            .into_iter()
            .map(|todo| Todo {
                id: todo.id,
                title: todo.title,
                completed: todo.completed != 0,
            })
            .collect();

        Ok(todos)
    }

    pub async fn create(&self, title: &str) -> Result<Todo, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO todos (title, completed) VALUES (?, ?)",
            title,
            false
        )
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        Ok(Todo {
            id: id,
            title: title.to_string(),
            completed: false,
        })
    }

    pub async fn update_completed(&self, id: i64, completed: bool) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE todos SET completed = ? WHERE id = ?", completed, id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM todos WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
