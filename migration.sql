CREATE TABLE IF NOT EXISTS items (
    id SERIAL PRIMARY KEY,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'in_progress' CHECK (status IN ('in_progress', 'done')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);