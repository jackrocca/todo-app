-- Initial todo app schema with advanced features
CREATE TABLE todos (
    id TEXT PRIMARY KEY NOT NULL,
    text TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    category TEXT,
    tags TEXT, -- JSON string of tags array
    priority TEXT CHECK (priority IN ('high', 'medium', 'low')),
    due_date DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX idx_todos_completed ON todos(completed);
CREATE INDEX idx_todos_category ON todos(category);
CREATE INDEX idx_todos_priority ON todos(priority);
CREATE INDEX idx_todos_due_date ON todos(due_date);
CREATE INDEX idx_todos_created_at ON todos(created_at);