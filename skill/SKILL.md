---
name: elastic-claude
description: Index and search project knowledge. Use when starting a new task to find related prior work, when ingesting documents or chats, or when searching project history for context.
---

# Elastic-Claude

Local search infrastructure for project knowledge.

## Config

Read connection from `~/.elastic-claude/config.yaml`:

```yaml
database:
  host: localhost
  port: 5433
  name: elastic_claude
  user: postgres
  password: elastic
```

## Schema

See [references/schema.md](references/schema.md)

## Ingest

Read files, extract metadata based on content/path, insert into entries table.

Example insert:
```sql
INSERT INTO entries (entry_type, content, metadata, file_path)
VALUES ('document', $content, $metadata::jsonb, $path);
```

## Search

Use tsvector full-text search, return ranked results with snippets:

```sql
SELECT id, entry_type, file_path, metadata,
       ts_headline('english', content, query) as snippet,
       ts_rank(content_tsv, query) as rank
FROM entries, to_tsquery('english', $search_terms) query
WHERE content_tsv @@ query
ORDER BY rank DESC
LIMIT 10;
```
