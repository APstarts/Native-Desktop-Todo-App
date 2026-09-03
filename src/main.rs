mod db;
mod models;
use dotenv::dotenv;
use slint::{ComponentHandle, Model, VecModel};
use sqlx::sqlite::SqlitePoolOptions;
use std::rc::Rc;

use crate::db::todos::TodoRepository;

slint::include_modules!();
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let pool = SqlitePoolOptions::new()
        .connect("sqlite://todos.db")
        .await?;
    let repository = TodoRepository::new(pool.clone());

    let database_todos = repository.get_all().await?;

    let slint_todos: Vec<Todo> = database_todos
        .into_iter()
        .map(|todo| Todo {
            id: todo.id as i32,
            title: todo.title.into(),
            completed: todo.completed,
        })
        .collect();

    let main_window = MainWindow::new()?;
    let todos = Rc::new(VecModel::from(slint_todos));
    main_window.set_todos(todos.into());
    let main_window_weak = main_window.as_weak();

    main_window.on_add_todo({
        let repository = repository.clone();
        let main_window_weak = main_window_weak.clone();

        move |title| {
            let repository = repository.clone();
            let main_window_weak = main_window_weak.clone();
            let title = title.to_string();
            tokio::spawn(async move {
                let repository = repository.clone();
                match repository.create(&title).await {
                    //making database operation here for adding
                    //todo
                    Ok(todo) => {
                        //once the database operation is successfull we move ahead
                        //with updating the ui with the new todo.
                        let result = main_window_weak.upgrade_in_event_loop(move |window| {
                            let model = window.get_todos();
                            let todos = model
                                .as_any()
                                .downcast_ref::<VecModel<Todo>>()
                                .expect("todos model should be a VecModel<Todo>");

                            todos.push(Todo {
                                id: todo.id as i32,
                                title: todo.title.into(),
                                completed: todo.completed,
                            });
                        });
                        if let Err(error) = result {
                            eprintln!("Failed to update UI after creating todo: {error}");
                        }
                    }
                    Err(error) => {
                        eprintln!("Failed to create todo: {error}")
                    }
                }
            });
        }
    });

    main_window.on_delete_todo({
        let repository = repository.clone();
        let window_weak = main_window.as_weak();

        move |id| {
            let repository = repository.clone();
            let window_weak = window_weak.clone();

            tokio::spawn(async move {
                match repository.delete(id as i64).await {
                    Ok(()) => {
                        let result = window_weak.upgrade_in_event_loop(move |window| {
                            let model = window.get_todos();

                            let todos = model
                                .as_any()
                                .downcast_ref::<VecModel<Todo>>()
                                .expect("todos model should be a VecModel<Todo>");

                            for index in 0..todos.row_count() {
                                if let Some(todo) = todos.row_data(index) {
                                    if todo.id == id {
                                        todos.remove(index);
                                        break;
                                    }
                                }
                            }
                        });

                        if let Err(error) = result {
                            eprintln!("Failed to update UI after deletion: {error}");
                        }
                    }

                    Err(error) => {
                        eprintln!("Failed to delete todo: {error}");
                    }
                }
            });
        }
    });

    main_window.on_update_todo({
        let repository = repository.clone();
        let window_weak = main_window.as_weak();

        move |id, completed| {
            let repository = repository.clone();
            let window_weak = window_weak.clone();

            tokio::spawn(async move {
                match repository.update_completed(id as i64, completed).await {
                    Ok(()) => {
                        let result = window_weak.upgrade_in_event_loop(move |window| {
                            let model = window.get_todos();

                            let todos = model
                                .as_any()
                                .downcast_ref::<VecModel<Todo>>()
                                .expect("todos model should be a VecModel<Todo>");

                            for index in 0..todos.row_count() {
                                if let Some(mut todo) = todos.row_data(index) {
                                    if todo.id == id {
                                        todo.completed = completed;

                                        todos.set_row_data(index, todo);

                                        break;
                                    }
                                }
                            }
                        });

                        if let Err(error) = result {
                            eprintln!("Failed to update UI: {error}");
                        }
                    }

                    Err(error) => {
                        eprintln!("Failed to update todo: {error}");
                    }
                }
            });
        }
    });

    main_window.run()?;

    Ok(())
}
