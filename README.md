# 🦀 Rust Todo App

A simple, fast, and reliable todo application built with Rust and deployed on DigitalOcean. Features a clean web interface and RESTful API.

## 🌟 Features

### Core Functionality
- **Add Todos**: Create new todo items with rich metadata
- **Toggle Completion**: Mark todos as complete or incomplete with visual feedback
- **Real-time Updates**: Dynamic web interface with instant feedback

### Advanced Organization
- **Categories**: Organize todos with customizable categories
- **Tags**: Flexible tagging system with comma-separated tags
- **Priority Levels**: Set High, Medium, or Low priority with visual indicators
- **Due Dates**: Schedule todos with datetime-based due dates

### Security & Authentication
- **User Authentication**: Secure JWT-based login and registration system
- **Protected Routes**: User-specific todo management with authorization
- **Password Hashing**: bcrypt-secured password storage
- **Session Management**: Persistent login with localStorage tokens

### Production Features
- **SQLite Database**: Persistent data storage with migrations
- **HTTPS Support**: Optional TLS/SSL encryption for secure connections
- **RESTful API**: Comprehensive JSON endpoints for programmatic access
- **Responsive Design**: Modern UI that works across devices
- **Service Management**: systemd integration for production deployment

## 🛠️ Tech Stack

- **Backend**: Rust with Axum web framework
- **Database**: SQLite with SQLX for async queries and migrations
- **Authentication**: JWT tokens with bcrypt password hashing
- **Frontend**: Vanilla JavaScript with modern ES6+ features
- **Security**: HTTPS/TLS support with rustls
- **Deployment**: DigitalOcean Droplet with systemd service management
- **Concurrency**: Tokio async runtime for high-performance I/O

## 🚀 Live Demo

**Access the app**: [http://143.110.212.251:3000](http://143.110.212.251:3000)

## 📋 API Endpoints

### Public Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Web interface |
| `POST` | `/auth/register` | User registration |
| `POST` | `/auth/login` | User authentication |

### Protected Endpoints (Require Authorization Header)
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/todos` | List user's todos (JSON) |
| `POST` | `/todos` | Create new todo with categories, tags, priority, due date |
| `POST` | `/toggle/:id` | Toggle todo completion |
| `GET` | `/categories` | List user's categories |

### API Usage Examples

```bash
# Register a new user
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "user1", "email": "user1@example.com", "password": "securepass"}'

# Login and get token
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "user1", "password": "securepass"}'

# Create a todo with metadata (replace JWT_TOKEN)
curl -X POST http://localhost:3000/todos \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer JWT_TOKEN" \
  -d '{"text": "Learn Rust", "category": "Education", "tags": ["programming", "rust"], "priority": "high", "due_date": "2025-12-31T23:59:59Z"}'

# Get all todos
curl http://localhost:3000/todos \
  -H "Authorization: Bearer JWT_TOKEN"

# Toggle todo completion
curl -X POST http://localhost:3000/toggle/todo-id \
  -H "Authorization: Bearer JWT_TOKEN"
```

## 💻 Development

### Prerequisites

- Rust 2024 edition or later
- Cargo package manager

### Dependencies

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
jsonwebtoken = "9.0"
bcrypt = "0.15"
async-trait = "0.1"
rustls = "0.21"
rustls-pemfile = "1.0"
tokio-rustls = "0.24"
```

### Local Development

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd todo-app
   ```

2. **Run the application**
   ```bash
   cargo run
   ```

3. **Access locally**
   - Web interface: http://localhost:3000
   - API: http://localhost:3000/todos

4. **Build for production**
   ```bash
   cargo build --release
   ```

## 🚀 Deployment

### DigitalOcean Droplet Setup

1. **Build release binary**
   ```bash
   cargo build --release
   ```

2. **Open firewall port**
   ```bash
   sudo ufw allow 3000
   ```

3. **Create systemd service**
   ```bash
   sudo systemctl enable todo-app.service
   sudo systemctl start todo-app.service
   ```

4. **Check service status**
   ```bash
   sudo systemctl status todo-app.service
   ```

### Service Management

```bash
# Check service status
sudo systemctl status todo-app

# Restart the service
sudo systemctl restart todo-app

# View service logs
sudo journalctl -u todo-app -f

# Stop the service
sudo systemctl stop todo-app
```

### HTTPS Deployment

For production deployment with HTTPS:

1. **Generate SSL certificates** (Let's Encrypt recommended)
   ```bash
   # Install certbot
   sudo apt install certbot
   
   # Generate certificates
   sudo certbot certonly --standalone -d yourdomain.com
   ```

2. **Use HTTPS service configuration**
   ```bash
   sudo cp todo-app-https.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable todo-app-https
   sudo systemctl start todo-app-https
   ```

3. **Environment variables for HTTPS**
   ```bash
   export USE_HTTPS=true
   export PORT=443
   export CERT_PATH=/etc/letsencrypt/live/yourdomain.com/fullchain.pem
   export KEY_PATH=/etc/letsencrypt/live/yourdomain.com/privkey.pem
   ```

4. **Quick deployment script**
   ```bash
   ./deploy.sh
   ```

## 🏗️ Architecture

The application follows a simple client-server architecture:

- **Web Server**: Axum handles HTTP requests on port 3000
- **Data Storage**: In-memory Vec<Todo> wrapped in Arc<Mutex> for thread safety
- **Concurrency**: Tokio async runtime handles concurrent requests
- **Frontend**: Single-page application with embedded HTML/CSS/JavaScript

### Data Structure

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Todo {
    id: u32,
    text: String,
    completed: bool,
}
```

## 🔧 Configuration

The app runs on `0.0.0.0:3000` by default, making it accessible from external networks. To change the port, modify the bind address in `src/main.rs:44`:

```rust
let listener = TcpListener::bind("0.0.0.0:YOUR_PORT").await.unwrap();
```

## 🛡️ Security Considerations

- The app currently uses in-memory storage (data is lost on restart)
- No authentication or authorization implemented
- Suitable for development and simple use cases
- For production use, consider adding persistent storage and authentication

## 🚀 Development Roadmap

### Phase 1: Core Infrastructure (Priority 1)
- [ ] **SQLite Persistent Storage** - Replace in-memory storage with SQLite database for data persistence
- [ ] **User Authentication** - Secure login/register system with JWT tokens

### Phase 2: Enhanced Todo Management (Priority 2) 
- [ ] **Categories & Tags** - Organize todos with customizable categories and flexible tagging system
- [ ] **Modern UI Redesign** - Contemporary interface with intuitive navigation and responsive design

### Phase 3: Advanced Features (Priority 3)
- [ ] **Due Dates & Priorities** - Time-based scheduling with priority levels (High/Medium/Low)
- [ ] **HTTPS Support** - Secure SSL/TLS encryption for production deployment

### 🎯 Key Focus Areas
- **Efficiency**: Optimized database queries and minimal resource usage
- **User Experience**: Intuitive categorization and tagging workflow
- **Modern Design**: Clean, responsive UI following current design principles
- **Data Persistence**: Reliable storage that survives server restarts

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is open source and available under the [MIT License](LICENSE).

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum) web framework
- Powered by [Tokio](https://tokio.rs/) async runtime
- Deployed on [DigitalOcean](https://www.digitalocean.com/)
- Created with assistance from [Claude Code](https://claude.ai/code)