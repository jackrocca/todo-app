use axum::{
    http::StatusCode,
    middleware,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;

mod simple_auth;
mod simple_db;
mod https;
use simple_auth::{AuthService, LoginRequest, RegisterRequest};
use simple_db::{Database, NewTodo, Todo};

#[tokio::main]
async fn main() {
    // Initialize SQLite database
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:todos.db".to_string());
    let db = Database::new(&database_url).await.expect("Failed to initialize database");
    let db = Arc::new(db);

    // Initialize auth service
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let auth_service = Arc::new(AuthService::new(db.get_pool().clone(), jwt_secret));

    // Public routes
    let public_routes = Router::new()
        .route("/", get(home))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login));

    // Protected routes
    let protected_routes = Router::new()
        .route("/todos", get(get_todos))
        .route("/todos", post(add_todo))
        .route("/toggle/:id", post(toggle_todo))
        .route("/categories", get(get_categories))
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            simple_auth::auth_middleware,
        ));

    let app = public_routes
        .merge(protected_routes)
        .with_state((db, auth_service));

    // Check for HTTPS configuration
    let use_https = std::env::var("USE_HTTPS").unwrap_or_else(|_| "false".to_string()) == "true";
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    
    if use_https {
        let cert_path = std::env::var("CERT_PATH").unwrap_or_else(|_| "cert.pem".to_string());
        let key_path = std::env::var("KEY_PATH").unwrap_or_else(|_| "key.pem".to_string());
        
        match https::load_tls_config(&cert_path, &key_path) {
            Ok(tls_config) => {
                let tls_acceptor = https::create_tls_acceptor(tls_config);
                let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
                
                println!("Todo app running on https://0.0.0.0:{}", port);
                println!("Database: {}", database_url);
                println!("TLS Certificate: {}", cert_path);
                println!("TLS Private Key: {}", key_path);
                
                loop {
                    let (stream, _) = listener.accept().await.unwrap();
                    let _tls_stream = tls_acceptor.accept(stream).await.unwrap();
                    let app = app.clone();

                    tokio::spawn(async move {
                        let _ = axum::serve(tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap(), app).await;
                    });
                }
            }
            Err(e) => {
                eprintln!("Failed to load TLS configuration: {}", e);
                https::generate_self_signed_cert().unwrap();
                std::process::exit(1);
            }
        }
    } else {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
        println!("Todo app running on http://0.0.0.0:{}", port);
        println!("Database: {}", database_url);
        println!("Note: To enable HTTPS, set USE_HTTPS=true with CERT_PATH and KEY_PATH");
        
        axum::serve(listener, app).await.unwrap();
    }
}

async fn home() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Rust Todo App</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
            .todo-item { margin: 10px 0; padding: 15px; border: 1px solid #ddd; border-radius: 8px; background: #f9f9f9; }
            .completed { text-decoration: line-through; opacity: 0.6; }
            .todo-meta { font-size: 12px; color: #666; margin-top: 5px; }
            .priority-high { border-left: 4px solid #dc3545; }
            .priority-medium { border-left: 4px solid #ffc107; }
            .priority-low { border-left: 4px solid #28a745; }
            .tag { background: #e9ecef; padding: 2px 6px; border-radius: 12px; font-size: 11px; margin-right: 4px; }
            input[type="text"], input[type="email"], input[type="password"], input[type="datetime-local"], select { 
                width: 200px; padding: 8px; margin: 5px; border: 1px solid #ddd; border-radius: 4px; 
            }
            button { padding: 8px 15px; margin: 5px; cursor: pointer; border: none; border-radius: 4px; }
            .toggle-btn { background: #007bff; color: white; }
            .add-btn { background: #28a745; color: white; }
            .danger-btn { background: #dc3545; color: white; }
            #loginSection, #registerSection, #todoSection { margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 8px; }
        </style>
    </head>
    <body>
        <h1>🦀 Rust Todo App</h1>
        
        <div id="loginSection">
            <h2>Login</h2>
            <input type="text" id="usernameInput" placeholder="Username">
            <input type="password" id="passwordInput" placeholder="Password">
            <button class="add-btn" onclick="login()">Login</button>
            <button class="toggle-btn" onclick="showRegister()">Register</button>
        </div>

        <div id="registerSection" style="display:none;">
            <h2>Register</h2>
            <input type="text" id="regUsernameInput" placeholder="Username">
            <input type="email" id="regEmailInput" placeholder="Email">
            <input type="password" id="regPasswordInput" placeholder="Password">
            <button class="add-btn" onclick="register()">Register</button>
            <button class="toggle-btn" onclick="showLogin()">Back to Login</button>
        </div>

        <div id="todoSection" style="display:none;">
            <h2>Todo Management</h2>
            <button class="toggle-btn" onclick="logout()">Logout</button>
            
            <div>
                <input type="text" id="todoInput" placeholder="Enter a new todo...">
                <input type="text" id="categoryInput" placeholder="Category (optional)">
                <input type="text" id="tagsInput" placeholder="Tags (comma-separated)">
                <select id="prioritySelect">
                    <option value="">Select Priority</option>
                    <option value="high">High</option>
                    <option value="medium">Medium</option>
                    <option value="low">Low</option>
                </select>
                <input type="datetime-local" id="dueDateInput" placeholder="Due date">
                <button class="add-btn" onclick="addTodo()">Add Todo</button>
            </div>

            <div>
                <label>Filter by category:</label>
                <select id="categoryFilter" onchange="loadTodos()">
                    <option value="">All Categories</option>
                </select>
            </div>
        </div>
        
        <div id="todos"></div>

        <script>
            let authToken = localStorage.getItem('authToken');

            // Authentication functions
            async function login() {
                const username = document.getElementById('usernameInput').value.trim();
                const password = document.getElementById('passwordInput').value.trim();
                if (!username || !password) return;

                try {
                    const response = await fetch('/auth/login', {
                        method: 'POST',
                        headers: {'Content-Type': 'application/json'},
                        body: JSON.stringify({username, password})
                    });

                    if (response.ok) {
                        const data = await response.json();
                        authToken = data.token;
                        localStorage.setItem('authToken', authToken);
                        showTodoSection();
                        loadTodos();
                        loadCategories();
                    } else {
                        alert('Login failed!');
                    }
                } catch (error) {
                    alert('Login error: ' + error.message);
                }
            }

            async function register() {
                const username = document.getElementById('regUsernameInput').value.trim();
                const email = document.getElementById('regEmailInput').value.trim();
                const password = document.getElementById('regPasswordInput').value.trim();
                if (!username || !email || !password) return;

                try {
                    const response = await fetch('/auth/register', {
                        method: 'POST',
                        headers: {'Content-Type': 'application/json'},
                        body: JSON.stringify({username, email, password})
                    });

                    if (response.ok) {
                        const data = await response.json();
                        authToken = data.token;
                        localStorage.setItem('authToken', authToken);
                        showTodoSection();
                        loadTodos();
                        loadCategories();
                    } else {
                        alert('Registration failed!');
                    }
                } catch (error) {
                    alert('Registration error: ' + error.message);
                }
            }

            function logout() {
                authToken = null;
                localStorage.removeItem('authToken');
                showLoginSection();
            }

            function showRegister() {
                document.getElementById('loginSection').style.display = 'none';
                document.getElementById('registerSection').style.display = 'block';
            }

            function showLogin() {
                document.getElementById('registerSection').style.display = 'none';
                document.getElementById('loginSection').style.display = 'block';
            }

            function showTodoSection() {
                document.getElementById('loginSection').style.display = 'none';
                document.getElementById('registerSection').style.display = 'none';
                document.getElementById('todoSection').style.display = 'block';
            }

            function showLoginSection() {
                document.getElementById('todoSection').style.display = 'none';
                document.getElementById('loginSection').style.display = 'block';
                document.getElementById('registerSection').style.display = 'none';
            }

            // Todo functions
            async function loadTodos() {
                if (!authToken) return;

                try {
                    const response = await fetch('/todos', {
                        headers: {'Authorization': `Bearer ${authToken}`}
                    });

                    if (response.ok) {
                        const todos = await response.json();
                        const todosDiv = document.getElementById('todos');
                        todosDiv.innerHTML = todos.map(todo => renderTodo(todo)).join('');
                    }
                } catch (error) {
                    console.error('Failed to load todos:', error);
                }
            }

            function renderTodo(todo) {
                const tags = todo.tags ? JSON.parse(todo.tags) : [];
                const tagHtml = tags.map(tag => `<span class="tag">${tag}</span>`).join('');
                const priorityClass = todo.priority ? `priority-${todo.priority}` : '';
                const dueDate = todo.due_date ? new Date(todo.due_date).toLocaleDateString() : '';
                
                return `
                    <div class="todo-item ${todo.completed ? 'completed' : ''} ${priorityClass}">
                        <div>
                            <strong>${todo.text}</strong>
                            <button class="toggle-btn" onclick="toggleTodo('${todo.id}')">
                                ${todo.completed ? 'Undo' : 'Complete'}
                            </button>
                        </div>
                        <div class="todo-meta">
                            ${todo.category ? `Category: ${todo.category} | ` : ''}
                            ${todo.priority ? `Priority: ${todo.priority} | ` : ''}
                            ${dueDate ? `Due: ${dueDate} | ` : ''}
                            Created: ${new Date(todo.created_at).toLocaleDateString()}
                        </div>
                        <div>${tagHtml}</div>
                    </div>
                `;
            }

            async function addTodo() {
                if (!authToken) return;

                const text = document.getElementById('todoInput').value.trim();
                if (!text) return;

                const category = document.getElementById('categoryInput').value.trim() || null;
                const tagsInput = document.getElementById('tagsInput').value.trim();
                const tags = tagsInput ? tagsInput.split(',').map(t => t.trim()) : null;
                const priority = document.getElementById('prioritySelect').value || null;
                const dueDateInput = document.getElementById('dueDateInput').value;
                const due_date = dueDateInput ? new Date(dueDateInput).toISOString() : null;

                try {
                    const response = await fetch('/todos', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            'Authorization': `Bearer ${authToken}`
                        },
                        body: JSON.stringify({text, category, tags, priority, due_date})
                    });

                    if (response.ok) {
                        document.getElementById('todoInput').value = '';
                        document.getElementById('categoryInput').value = '';
                        document.getElementById('tagsInput').value = '';
                        document.getElementById('prioritySelect').value = '';
                        document.getElementById('dueDateInput').value = '';
                        loadTodos();
                        loadCategories();
                    }
                } catch (error) {
                    console.error('Failed to add todo:', error);
                }
            }

            async function toggleTodo(id) {
                if (!authToken) return;

                try {
                    await fetch(`/toggle/${id}`, {
                        method: 'POST',
                        headers: {'Authorization': `Bearer ${authToken}`}
                    });
                    loadTodos();
                } catch (error) {
                    console.error('Failed to toggle todo:', error);
                }
            }

            async function loadCategories() {
                if (!authToken) return;

                try {
                    const response = await fetch('/categories', {
                        headers: {'Authorization': `Bearer ${authToken}`}
                    });

                    if (response.ok) {
                        const categories = await response.json();
                        const select = document.getElementById('categoryFilter');
                        select.innerHTML = '<option value="">All Categories</option>';
                        categories.forEach(cat => {
                            select.innerHTML += `<option value="${cat}">${cat}</option>`;
                        });
                    }
                } catch (error) {
                    console.error('Failed to load categories:', error);
                }
            }

            // Initialize app
            if (authToken) {
                showTodoSection();
                loadTodos();
                loadCategories();
            } else {
                showLoginSection();
            }
        </script>
    </body>
    </html>
    "#)
}

async fn register(
    axum::extract::State((_, auth_service)): axum::extract::State<(Arc<Database>, Arc<AuthService>)>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<simple_auth::AuthResponse>, StatusCode> {
    match auth_service.register(req).await {
        Ok(response) => Ok(Json(response)),
        Err(err) => Err(err.into()),
    }
}

async fn login(
    axum::extract::State((_, auth_service)): axum::extract::State<(Arc<Database>, Arc<AuthService>)>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<simple_auth::AuthResponse>, StatusCode> {
    match auth_service.login(req).await {
        Ok(response) => Ok(Json(response)),
        Err(err) => Err(err.into()),
    }
}

async fn get_todos(
    axum::extract::State((db, _)): axum::extract::State<(Arc<Database>, Arc<AuthService>)>,
    axum::Extension(user_id): axum::Extension<String>,
) -> Result<Json<Vec<Todo>>, StatusCode> {
    match db.get_todos(Some(&user_id)).await {
        Ok(todos) => Ok(Json(todos)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn add_todo(
    axum::extract::State((db, _)): axum::extract::State<(Arc<Database>, Arc<AuthService>)>,
    axum::Extension(user_id): axum::Extension<String>,
    Json(new_todo): Json<NewTodo>,
) -> StatusCode {
    match db.create_todo(new_todo, Some(&user_id)).await {
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn toggle_todo(
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::State((db, _)): axum::extract::State<(Arc<Database>, Arc<AuthService>)>,
) -> StatusCode {
    match db.toggle_todo(&id).await {
        Ok(Some(_)) => StatusCode::OK,
        Ok(None) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_categories(
    axum::extract::State((db, _)): axum::extract::State<(Arc<Database>, Arc<AuthService>)>,
) -> Result<Json<Vec<String>>, StatusCode> {
    match db.get_categories().await {
        Ok(categories) => Ok(Json(categories)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
