-- elastic-claude database schema

CREATE TABLE entries (
    id SERIAL PRIMARY KEY,
    entry_type TEXT NOT NULL,
    content TEXT NOT NULL,
    content_tsv TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', content)) STORED,
    blob BYTEA,
    mime_type TEXT,
    metadata JSONB DEFAULT '{}',
    source_id INT REFERENCES entries(id),
    file_path TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Full-text search index
CREATE INDEX idx_content_tsv ON entries USING GIN(content_tsv);

-- JSONB metadata index for filtering
CREATE INDEX idx_metadata ON entries USING GIN(metadata);

-- Entry type index for filtering by type
CREATE INDEX idx_entry_type ON entries(entry_type);

-- File path index for deduplication
CREATE INDEX idx_file_path ON entries(file_path);

-- Source relationship index
CREATE INDEX idx_source_id ON entries(source_id);
