# 🦀 Rust Todo App

A simple, fast, and reliable todo application built with Rust and deployed on DigitalOcean. Features a clean web interface and RESTful API.

## 🌟 Features

- **Add Todos**: Create new todo items with a simple text input
- **Toggle Completion**: Mark todos as complete or incomplete
- **Real-time Updates**: Dynamic web interface with instant feedback
- **RESTful API**: JSON endpoints for programmatic access
- **Production Ready**: Deployed with systemd service management

## 🛠️ Tech Stack

- **Backend**: Rust with Axum web framework
- **Frontend**: Vanilla JavaScript with embedded HTML/CSS
- **Deployment**: DigitalOcean Droplet with systemd
- **Concurrency**: Tokio async runtime
- **Data**: In-memory storage with Arc<Mutex<Vec>>

## 🚀 Live Demo

**Access the app**: [http://143.110.212.251:3000](http://143.110.212.251:3000)

## 📋 API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Web interface |
| `GET` | `/todos` | List all todos (JSON) |
| `POST` | `/todos` | Create new todo |
| `POST` | `/toggle/:id` | Toggle todo completion |

### API Usage Examples

```bash
# Get all todos
curl http://143.110.212.251:3000/todos

# Add a new todo
curl -X POST http://143.110.212.251:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"text": "Learn Rust"}'

# Toggle todo completion (replace 1 with todo ID)
curl -X POST http://143.110.212.251:3000/toggle/1
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

## 📝 Future Enhancements

- [ ] Persistent storage (SQLite/PostgreSQL)
- [ ] User authentication
- [ ] Todo categories/tags
- [ ] Due dates and priorities
- [ ] API rate limiting
- [ ] HTTPS support
- [ ] Docker containerization

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