# elastic-claude

Local search infrastructure for project knowledge. Index documents, chat sessions, and code with PostgreSQL full-text search, then query them through Claude Code.

## Installation

### From source

```bash
cd cli
cargo install --path .
```

### From releases

Download the binary for your platform from [GitHub Releases](https://github.com/Kirill-Babkin/elastic-claude/releases).

## Quick Start

```bash
# Initialize (pulls PostgreSQL, creates database, installs skill)
elastic-claude init

# Add a document
elastic-claude add -t document -p ./docs/architecture.md -m '{"project": "my-project", "title": "Architecture"}'

# Search your knowledge base
elastic-claude search "authentication flow"

# Save the current Claude Code chat session
elastic-claude current-chat -m '{"project": "my-project", "title": "Session title"}'

# Check status
elastic-claude status
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize PostgreSQL container, database, config, and skill |
| `start` | Start the elastic-claude container |
| `stop` | Stop the container (preserves data) |
| `status` | Show container status, entry counts, and database size |
| `destroy` | Remove container (use `--include-data` to also remove data) |
| `add` | Add an entry to the knowledge base |
| `search <query>` | Search the knowledge base |
| `current-chat` | Ingest the current Claude Code session |
| `chat <file>` | Ingest a specific chat session file |
| `get <id>` | Retrieve an entry by ID |

### Add Command

```bash
# From file path (preferred - stores file_path in DB)
elastic-claude add -t <type> -p <file_path> [-m '<json_metadata>']

# Inline content
elastic-claude add -t <type> -c "<content>" [-m '<json_metadata>']

# From stdin
cat file.md | elastic-claude add -t <type> [-m '<json_metadata>']
```

### Current Chat Command

Auto-detects the current Claude Code session based on the most recently modified chat file in the project directory.

```bash
# Get path to current chat file
elastic-claude current-chat --path-only

# Ingest current chat with metadata
elastic-claude current-chat -m '{"project": "my-project", "title": "Session title", "tags": ["topic1"]}'
```

### Get Command

```bash
# Get full entry details
elastic-claude get <id>

# Get just the content
elastic-claude get <id> --content-only

# Show tsvector tokens (for debugging search)
elastic-claude get <id> --tsv
```

## Configuration

Config is stored at `~/.elastic-claude/config.yaml`:

```yaml
database:
  host: localhost
  port: 5433
  name: elastic_claude
  user: postgres
  password: elastic
```

## How It Works

1. **Storage**: PostgreSQL with full-text search (tsvector/tsquery)
2. **Indexing**: Documents are parsed and stored with metadata
3. **Search**: Full-text search with ranking and multi-fragment snippets
4. **Chat Extraction**: JSONL chat files are preprocessed to extract text and thinking content
5. **Integration**: Claude Code skill enables natural language queries

## Entry Types

- **document**: Markdown files, specs, notes
- **chat**: Claude Code conversation sessions (with thinking content)
- **code**: Code snippets or files

## Requirements

- Docker (for PostgreSQL)
- Rust (for building from source)

## License

MIT
