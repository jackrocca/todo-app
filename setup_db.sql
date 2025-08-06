-- Initial todo app schema with advanced features
CREATE TABLE todos (
    id TEXT PRIMARY KEY NOT NULL,
    text TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    category TEXT,
    tags TEXT,
    priority TEXT CHECK (priority IN ('high', 'medium', 'low')),
    due_date DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create users table for authentication
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add user_id to todos table to associate todos with users
ALTER TABLE todos ADD COLUMN user_id TEXT REFERENCES users(id);

-- Create indexes for better query performance
CREATE INDEX idx_todos_completed ON todos(completed);
CREATE INDEX idx_todos_category ON todos(category);
CREATE INDEX idx_todos_priority ON todos(priority);
CREATE INDEX idx_todos_due_date ON todos(due_date);
CREATE INDEX idx_todos_created_at ON todos(created_at);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_todos_user_id ON todos(user_id);