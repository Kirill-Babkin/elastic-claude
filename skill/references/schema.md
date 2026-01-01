# Database Schema Reference

## Table: entries

| Column | Type | Description |
|--------|------|-------------|
| id | SERIAL | Primary key |
| entry_type | TEXT | Type of entry (document, chat, summary) |
| content | TEXT | Main text content |
| content_tsv | TSVECTOR | Auto-generated search vector |
| blob | BYTEA | Binary data (optional) |
| mime_type | TEXT | MIME type for blob |
| metadata | JSONB | Flexible metadata |
| source_id | INT | Reference to parent entry |
| file_path | TEXT | Original file path |
| created_at | TIMESTAMP | Creation timestamp |

## Metadata Conventions

### document
```json
{
  "ticket_id": "PROJ-123",
  "doc_type": "spec",
  "title": "Feature Specification",
  "project": "my-project",
  "date": "2025-01-15"
}
```

### chat
```json
{
  "session_id": "abc123",
  "started_at": "2025-01-15T10:00:00Z",
  "ended_at": "2025-01-15T11:30:00Z",
  "topics": ["authentication", "api-design"]
}
```

### summary
```json
{
  "summary_type": "daily",
  "generated_at": "2025-01-15T23:00:00Z",
  "source_ids": [1, 2, 3]
}
```

## Example Queries

### Insert a document
```sql
INSERT INTO entries (entry_type, content, metadata, file_path)
VALUES (
  'document',
  'Full document content here...',
  '{"doc_type": "spec", "title": "API Design", "project": "elastic-claude"}',
  '/path/to/document.md'
);
```

### Search with ranking
```sql
SELECT
  id,
  entry_type,
  file_path,
  metadata,
  ts_headline('english', content, query, 'MaxWords=50') as snippet,
  ts_rank(content_tsv, query) as rank
FROM entries, to_tsquery('english', 'search & terms') query
WHERE content_tsv @@ query
ORDER BY rank DESC
LIMIT 10;
```

### Filter by metadata
```sql
SELECT * FROM entries
WHERE metadata->>'project' = 'elastic-claude'
  AND entry_type = 'document';
```

### Get related entries
```sql
SELECT * FROM entries
WHERE source_id = 123
   OR id = 123;
```
