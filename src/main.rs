use dotenv::dotenv;
use slint::{Model, VecModel};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::rc::Rc;
use std::str::FromStr;

slint::include_modules!();
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let connection_options =
        SqliteConnectOptions::from_str("sqlite://todos.db")?.create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .connect_with(connection_options)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    println!("Database ready");
    // sqlx::query("INSERT INTO todos (title, completed) VALUES (?, ?);")
    //     .bind("Learn Rust")
    //     .bind(false)
    //     .execute(&pool)
    //     .await?;
    // println!("Todo inserted!");
    let rows = sqlx::query!("SELECT id, title, COMPLETED FROM todos ORDER BY id;")
        .fetch_all(&pool)
        .await?;

    let slint_todos: Vec<Todo> = rows
        .into_iter()
        .map(|row| Todo {
            id: row.id as i32,
            title: row.title.into(),
            completed: row.COMPLETED != 0,
        })
        .collect();

    let todos = Rc::new(VecModel::from(slint_todos));

    let main_window = MainWindow::new()?;
    main_window.set_todos(todos.clone().into());
    main_window.on_add_todo({
        let pool = pool.clone();
        let todos = todos.clone();

        move |title| {
            let pool = pool.clone();
            let todos = todos.clone();

            slint::spawn_local(async move {
                let result = sqlx::query("INSERT INTO todos (title, COMPLETED) VALUES (?,?);")
                    .bind(title.as_str())
                    .bind(false)
                    .execute(&pool)
                    .await;

                match result {
                    Ok(query_result) => {
                        todos.push(Todo {
                            id: query_result.last_insert_rowid() as i32,
                            title: title,
                            completed: false,
                        });
                    }
                    Err(error) => {
                        eprintln!("Failed to insert todo: {error}");
                    }
                }
            })
            .unwrap();
        }
    });

    //deleting a todo
    main_window.on_delete_todo({
        let pool = pool.clone();
        let todos = todos.clone();
        move |id, index| {
            let pool = pool.clone();
            let todos = todos.clone();
            slint::spawn_local(async move {
                let result = sqlx::query("DELETE FROM todos WHERE id = ?;")
                    .bind(id)
                    .execute(&pool)
                    .await;

                match result {
                    Ok(_) => {
                        todos.remove(index as usize);
                    }
                    Err(error) => {
                        eprintln!("Failed to delete todo: {error}");
                    }
                }
            })
            .unwrap();
        }
    });

    //update a todo for the checkbox checked
    main_window.on_update_todo({
        let pool = pool.clone();
        let todos = todos.clone();
        move |id, index, completed| {
            let pool = pool.clone();
            let todos = todos.clone();
            slint::spawn_local(async move {
                let result = sqlx::query("UPDATE todos SET COMPLETED = ? WHERE id = ?;")
                    .bind(completed)
                    .bind(id)
                    .execute(&pool)
                    .await;
                match result {
                    Ok(_) => {
                        for i in 0..todos.row_count() {
                            if let Some(mut todo) = todos.row_data(i) {
                                if todo.id == id {
                                    todo.completed = completed;
                                    todos.set_row_data(index as usize, todo);
                                    break;
                                }
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("Failed to update todo: {error}");
                    }
                }
            })
            .unwrap();
        }
    });
    main_window.run()?;

    // let todos = Rc::new(VecModel::from(vec![
    //     // because VecModel already
    //     // provides interior mutability
    //     // that's why we didn't use RefCell
    //     // here.
    //     Todo {
    //         title: "learn slint".into(),
    //         completed: false,
    //     },
    //     Todo {
    //         title: "learn rust".into(),
    //         completed: false,
    //     },
    //     Todo {
    //         title: "build gui".into(),
    //         completed: false,
    //     },
    // ]));
    // todos.push(Todo {
    //     title: "This was added from rust".into(),
    //     completed: false,
    // });
    // // add a todo
    // main_window.on_add_todo({
    //     let todos = Rc::clone(&todos);
    //     move |text| {
    //         todos.push(Todo {
    //             title: text.into(),
    //             completed: false,
    //         });
    //     }
    // });
    // main_window.set_todos(todos.clone().into());
    // // delete a todo
    // main_window.on_delete_todo({
    //     let todos = Rc::clone(&todos);
    //     move |index| {
    //         todos.remove(index as usize);
    //     }
    // });
    // // updating a todo with the checked mark
    // main_window.on_update_todo({
    //     let todos = Rc::clone(&todos);
    //     move |index, completed| {
    //         let mut todo = todos.row_data(index as usize).unwrap(); //getting the row
    //         //using the index
    //         todo.completed = completed; //updating the completed key's value with the boolean
    //         //received from the ui
    //         println!("Updating todo {index}: completed = {completed}");
    //     }
    // });
    // main_window.run()
    Ok(())
}
