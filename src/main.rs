mod app;
mod db;
mod models;
use std::rc::Rc;

use dotenv::dotenv;
use sqlx::sqlite::SqlitePoolOptions;

use crate::db::todos::TodoRepository;

slint::include_modules!();
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let pool = SqlitePoolOptions::new()
        .connect("sqlite://todos.db")
        .await?;
    let repository = TodoRepository::new(pool.clone());

    let state = Rc::new(app::state::State::new(repository).await?);

    let main_window = MainWindow::new()?;

    main_window.set_todos(state.todos.clone().into());

    main_window.on_delete_todo({
        let state = Rc::clone(&state);
        move |id| {
            let state = state.clone();
            slint::spawn_local(async move {
                if let Err(error) = state.delete_todo(id).await {
                    eprintln!("Delete failed: {error}");
                }
            })
            .unwrap();
        }
    });

    main_window.on_add_todo({
        let state = state.clone();

        move |title| {
            let state = state.clone();
            slint::spawn_local(async move {
                if let Err(error) = state.add_todo(&title).await {
                    eprintln!("Add failed: {error}");
                }
            })
            .unwrap();
        }
    });

    main_window.on_update_todo({
        let state = state.clone();

        move |id, completed| {
            let state = state.clone();
            slint::spawn_local(async move {
                if let Err(error) = state.update_todo(id, completed).await {
                    eprintln!("Failed to update todo: {error}");
                }
            })
            .unwrap();
        }
    });

    main_window.run()?;

    Ok(())
}
