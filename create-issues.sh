#!/bin/bash
# Run this script after authenticating with: gh auth login

REPO="Kirill-Babkin/elastic-claude"

# First, create the labels
echo "Creating labels..."
gh label create "phase-1" --repo "$REPO" --color "0E8A16" --description "Phase 1: Setup" 2>/dev/null || true
gh label create "phase-2" --repo "$REPO" --color "1D76DB" --description "Phase 2: CLI Commands" 2>/dev/null || true
gh label create "phase-3" --repo "$REPO" --color "5319E7" --description "Phase 3: Claude Integration" 2>/dev/null || true
gh label create "phase-4" --repo "$REPO" --color "FBCA04" --description "Phase 4: Skill" 2>/dev/null || true
gh label create "phase-5" --repo "$REPO" --color "D93F0B" --description "Phase 5: Polish" 2>/dev/null || true
gh label create "setup" --repo "$REPO" --color "C5DEF5" --description "Setup tasks" 2>/dev/null || true
gh label create "database" --repo "$REPO" --color "BFD4F2" --description "Database related" 2>/dev/null || true
gh label create "cli" --repo "$REPO" --color "D4C5F9" --description "CLI functionality" 2>/dev/null || true
gh label create "docker" --repo "$REPO" --color "0052CC" --description "Docker related" 2>/dev/null || true
gh label create "claude-integration" --repo "$REPO" --color "7057FF" --description "Claude Code integration" 2>/dev/null || true
gh label create "skill" --repo "$REPO" --color "FEF2C0" --description "Claude skill" 2>/dev/null || true
gh label create "polish" --repo "$REPO" --color "EDEDED" --description "Polish and cleanup" 2>/dev/null || true
gh label create "docs" --repo "$REPO" --color "0075CA" --description "Documentation" 2>/dev/null || true
gh label create "distribution" --repo "$REPO" --color "006B75" --description "Distribution and releases" 2>/dev/null || true

echo "Creating issues..."

# Issue 1
gh issue create --repo "$REPO" \
  --title "Initialize Rust project structure" \
  --label "phase-1,setup" \
  --body 'Set up the basic Rust project structure:

```
elastic-claude/
├── cli/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── skill/
│   ├── SKILL.md
│   └── references/
│       └── schema.md
├── docker/
│   └── init.sql
└── README.md
```

Add dependencies to `cli/Cargo.toml`:

- `clap` - CLI parsing
- `bollard` - Docker API
- `tokio` - async runtime
- `tokio-postgres` - database
- `serde` / `serde_yaml` - config
- `dirs` - home directory resolution'

# Issue 2
gh issue create --repo "$REPO" \
  --title "Create database schema" \
  --label "phase-1,database" \
  --body 'Create `docker/init.sql` with the entries table:

```sql
CREATE TABLE entries (
    id SERIAL PRIMARY KEY,
    entry_type TEXT NOT NULL,
    content TEXT NOT NULL,
    content_tsv TSVECTOR GENERATED ALWAYS AS (to_tsvector('\''english'\'', content)) STORED,
    blob BYTEA,
    mime_type TEXT,
    metadata JSONB DEFAULT '\''{}'\'',
    source_id INT REFERENCES entries(id),
    file_path TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_content_tsv ON entries USING GIN(content_tsv);
CREATE INDEX idx_metadata ON entries USING GIN(metadata);
CREATE INDEX idx_entry_type ON entries(entry_type);
```'

# Issue 3
gh issue create --repo "$REPO" \
  --title "Implement elastic-claude init command" \
  --label "phase-2,cli,docker" \
  --body 'Implement the `init` command that sets up everything:

1. Pull `postgres:16-alpine` image
2. Create Docker volume `elastic-claude-data`
3. Start container:
   - Name: `elastic-claude-db`
   - Port: `5433:5432`
   - Env: `POSTGRES_PASSWORD`, `POSTGRES_DB`
4. Wait for PostgreSQL to be ready
5. Run `init.sql` migration
6. Copy skill to `~/.claude/skills/elastic-claude/`
7. Write config to `~/.elastic-claude/config.yaml`
8. Print success message

Config format:

```yaml
database:
  host: localhost
  port: 5433
  name: elastic_claude
  user: postgres
  password: elastic
```'

# Issue 4
gh issue create --repo "$REPO" \
  --title "Implement elastic-claude start command" \
  --label "phase-2,cli,docker" \
  --body 'Start an existing elastic-claude container if it'\''s stopped.

1. Check if container exists
2. If not, print error suggesting `elastic-claude init`
3. If exists but stopped, start it
4. If already running, print status'

# Issue 5
gh issue create --repo "$REPO" \
  --title "Implement elastic-claude stop command" \
  --label "phase-2,cli,docker" \
  --body 'Stop the running elastic-claude container.

- Keep the volume intact (data preserved)
- Handle case where container doesn'\''t exist
- Handle case where already stopped'

# Issue 6
gh issue create --repo "$REPO" \
  --title "Implement elastic-claude status command" \
  --label "phase-2,cli" \
  --body 'Show current status of elastic-claude:

- Container status (running/stopped/not found)
- Entry count by type (document, chat, summary, etc.)
- Database size
- Config file location

Example output:

```
Container: running (elastic-claude-db)
Database:  elastic_claude @ localhost:5433
Entries:   47 documents, 12 chats, 5 summaries
Size:      24 MB
Config:    ~/.elastic-claude/config.yaml
```'

# Issue 7
gh issue create --repo "$REPO" \
  --title "Implement elastic-claude destroy command" \
  --label "phase-2,cli,docker" \
  --body 'Clean up elastic-claude installation:

1. Stop and remove container
2. By default, keep volume (data preserved)
3. With `--include-data` flag, also remove volume
4. Remove config file
5. Optionally remove skill from `~/.claude/skills/`

Require confirmation before destroying.'

# Issue 8
gh issue create --repo "$REPO" \
  --title "Implement elastic-claude ingest command" \
  --label "phase-3,cli,claude-integration" \
  --body 'Ingest files by spawning Claude Code:

```bash
elastic-claude ingest ./docs/**/*.md
```

Implementation:

1. Resolve glob patterns to file list
2. Spawn Claude Code with prompt:
   ```
   Use the elastic-claude skill to ingest documents.
   Files: {resolved_paths}

   For each file:
     - Read content
     - Extract metadata based on content and path
     - Insert into database
   ```
3. Stream Claude'\''s output to terminal'

# Issue 9
gh issue create --repo "$REPO" \
  --title "Implement elastic-claude search command" \
  --label "phase-3,cli,claude-integration" \
  --body 'Search entries by spawning Claude Code:

```bash
elastic-claude search "pdf template lifecycle"
```

Implementation:

1. Spawn Claude Code with prompt:
   ```
   Use the elastic-claude skill to search for: {query}

   Return relevant entries with snippets and metadata.
   ```
2. Stream Claude'\''s output to terminal'

# Issue 10
gh issue create --repo "$REPO" \
  --title "Implement elastic-claude chat command" \
  --label "phase-3,cli,claude-integration" \
  --body 'Ingest a chat session:

```bash
elastic-claude chat ./session-2025-12-31.json
```

Implementation:

1. Spawn Claude Code with prompt:
   ```
   Use the elastic-claude skill to ingest this chat session.
   File: {session_file}

   Parse the conversation, extract topics and metadata,
   store as entry_type='\''chat'\''.
   ```
2. Stream Claude'\''s output to terminal'

# Issue 11
gh issue create --repo "$REPO" \
  --title "Create elastic-claude skill" \
  --label "phase-4,skill" \
  --body 'Create the Claude Code skill at `skill/SKILL.md`:

```markdown
---
name: elastic-claude
description: Index and search project knowledge. Use when starting
  a new task to find related prior work, when ingesting documents
  or chats, or when searching project history for context.
---

# Elastic-Claude

Local search infrastructure for project knowledge.

## Config
Read connection from `~/.elastic-claude/config.yaml`

## Schema
See [references/schema.md](references/schema.md)

## Ingest
Read files, extract metadata based on content/path, insert into entries table.

## Search
Use tsvector full-text search, return ranked results with snippets.
```

Keep it minimal—Claude figures out details from context.'

# Issue 12
gh issue create --repo "$REPO" \
  --title "Create schema reference for skill" \
  --label "phase-4,skill" \
  --body 'Create `skill/references/schema.md`:

- Document table structure
- Document metadata conventions per entry_type:
  - `document`: ticket_id, doc_type, title, project, date
  - `chat`: session_id, started_at, ended_at, topics
  - `summary`: summary_type, generated_at, source_ids
- Include example SQL queries for ingest and search'

# Issue 13
gh issue create --repo "$REPO" \
  --title "Add error handling" \
  --label "phase-5,polish" \
  --body 'Handle common error cases gracefully:

- Docker not installed → helpful error with install link
- Docker daemon not running → suggest starting Docker
- Port 5433 in use → suggest alternative or show what'\''s using it
- Database connection failed → check container status first
- Skill directory not writable → show permission error

All errors should suggest a fix, not just fail.'

# Issue 14
gh issue create --repo "$REPO" \
  --title "Write README" \
  --label "phase-5,docs" \
  --body 'Create comprehensive `README.md`:

- Project description (what problem it solves)
- Installation options:
  - `cargo install elastic-claude`
  - Download binary from releases
- Quick start guide
- Command reference
- Configuration options
- Example workflows
- Contributing section'

# Issue 15
gh issue create --repo "$REPO" \
  --title "Set up GitHub releases" \
  --label "phase-5,distribution" \
  --body 'Automate binary releases:

- GitHub Action to build on tag push
- Build targets:
  - macOS arm64
  - macOS x64
  - Linux x64
- Upload binaries to GitHub release
- Consider publishing to crates.io'

echo ""
echo "✅ All 15 issues created!"
echo ""
echo "To add them to a project, go to:"
echo "https://github.com/Kirill-Babkin/elastic-claude/projects"
