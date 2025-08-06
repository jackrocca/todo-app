use axum::{
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Todo {
    id: u32,
    text: String,
    completed: bool,
}

type TodoStore = Arc<Mutex<Vec<Todo>>>;

#[tokio::main]
async fn main() {
    let todos: TodoStore = Arc::new(Mutex::new(vec![
        Todo {
            id: 1,
            text: "Create a todo app".to_string(),
            completed: false,
        },
        Todo {
            id: 2,
            text: "Deploy it to the web".to_string(),
            completed: false,
        },
    ]));

    let app = Router::new()
        .route("/", get(home))
        .route("/todos", get(get_todos))
        .route("/todos", post(add_todo))
        .route("/toggle/:id", post(toggle_todo))
        .with_state(todos);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Todo app running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Rust Todo App</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
            .todo-item { margin: 10px 0; padding: 10px; border: 1px solid #ddd; border-radius: 5px; }
            .completed { text-decoration: line-through; opacity: 0.6; }
            input[type="text"] { width: 300px; padding: 8px; margin: 5px; }
            button { padding: 8px 15px; margin: 5px; cursor: pointer; }
            .toggle-btn { background: #007bff; color: white; border: none; border-radius: 3px; }
            .add-btn { background: #28a745; color: white; border: none; border-radius: 3px; }
        </style>
    </head>
    <body>
        <h1>🦀 Rust Todo App</h1>
        
        <div>
            <input type="text" id="todoInput" placeholder="Enter a new todo...">
            <button class="add-btn" onclick="addTodo()">Add Todo</button>
        </div>
        
        <div id="todos"></div>

        <script>
            async function loadTodos() {
                const response = await fetch('/todos');
                const todos = await response.json();
                const todosDiv = document.getElementById('todos');
                todosDiv.innerHTML = todos.map(todo => `
                    <div class="todo-item ${todo.completed ? 'completed' : ''}">
                        <span>${todo.text}</span>
                        <button class="toggle-btn" onclick="toggleTodo(${todo.id})">
                            ${todo.completed ? 'Undo' : 'Complete'}
                        </button>
                    </div>
                `).join('');
            }

            async function addTodo() {
                const input = document.getElementById('todoInput');
                const text = input.value.trim();
                if (!text) return;
                
                await fetch('/todos', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({text})
                });
                
                input.value = '';
                loadTodos();
            }

            async function toggleTodo(id) {
                await fetch(`/toggle/${id}`, {method: 'POST'});
                loadTodos();
            }

            loadTodos();
        </script>
    </body>
    </html>
    "#)
}

async fn get_todos(todos: axum::extract::State<TodoStore>) -> Json<Vec<Todo>> {
    let todos = todos.lock().unwrap();
    Json(todos.clone())
}

#[derive(Deserialize)]
struct NewTodo {
    text: String,
}

async fn add_todo(
    todos: axum::extract::State<TodoStore>,
    Json(new_todo): Json<NewTodo>,
) -> StatusCode {
    let mut todos = todos.lock().unwrap();
    let id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    todos.push(Todo {
        id,
        text: new_todo.text,
        completed: false,
    });
    StatusCode::CREATED
}

async fn toggle_todo(
    axum::extract::Path(id): axum::extract::Path<u32>,
    todos: axum::extract::State<TodoStore>,
) -> StatusCode {
    let mut todos = todos.lock().unwrap();
    if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
        todo.completed = !todo.completed;
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}
