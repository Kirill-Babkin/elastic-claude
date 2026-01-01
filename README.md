# elastic-claude

Local search infrastructure for project knowledge. Index documents, chat sessions, and summaries with PostgreSQL full-text search, then query them through Claude Code.

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

# Ingest documents
elastic-claude ingest "./docs/**/*.md"

# Search your knowledge base
elastic-claude search "authentication flow"

# Ingest a Claude Code chat session
elastic-claude chat ./session-2025-01-15.json

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
| `ingest <patterns>` | Ingest files matching glob patterns |
| `search <query>` | Search the knowledge base |
| `chat <file>` | Ingest a chat session file |

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
3. **Search**: Full-text search with ranking and snippets
4. **Integration**: Claude Code skill enables natural language queries

## Entry Types

- **document**: Markdown files, specs, notes
- **chat**: Claude Code conversation sessions
- **summary**: AI-generated summaries of related entries

## Requirements

- Docker
- Claude Code (for ingest/search/chat commands)

## License

MIT
