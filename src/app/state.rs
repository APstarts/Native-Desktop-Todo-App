use crate::Todo as SlintTodo;
use crate::db::todos::TodoRepository;
use slint::{Model, VecModel};
use std::rc::Rc;

pub struct State {
    pub todos: Rc<VecModel<SlintTodo>>,
    pub repository: TodoRepository,
}

impl State {
    pub async fn new(repository: TodoRepository) -> Result<Self, sqlx::Error> {
        let database_todos = repository.get_all().await?;
        let slint_todos: Vec<SlintTodo> = database_todos
            .into_iter()
            .map(|todo| SlintTodo {
                id: todo.id as i32,
                title: todo.title.into(),
                completed: todo.completed,
            })
            .collect();

        Ok(Self {
            todos: Rc::new(VecModel::from(slint_todos)),
            repository,
        })
    }

    pub async fn add_todo(&self, title: &str) -> Result<(), sqlx::Error> {
        let todo = self.repository.create(title).await?;

        self.todos.push(SlintTodo {
            id: todo.id as i32,
            title: todo.title.into(),
            completed: todo.completed,
        });

        Ok(())
    }

    pub async fn delete_todo(&self, id: i32) -> Result<(), sqlx::Error> {
        // step 1: we delete from the database
        self.repository.delete(id as i64).await?;

        //step 2: we update the ui state accordingly if the delete succeeds
        for index in 0..self.todos.row_count() {
            if let Some(todo) = self.todos.row_data(index) {
                if todo.id == id {
                    self.todos.remove(index);
                    break;
                }
            }
        }
        Ok(())
    }

    pub async fn update_todo(&self, id: i32, completed: bool) -> Result<(), sqlx::Error> {
        //updating the todo in the database
        self.repository
            .update_completed(id as i64, completed)
            .await?;

        //updating the todo in the UI state
        for index in 0..self.todos.row_count() {
            if let Some(mut todo) = self.todos.row_data(index) {
                if todo.id == id {
                    todo.completed = completed;
                    self.todos.set_row_data(index, todo);
                    break;
                }
            }
        }
        Ok(())
    }
}
