# AGENTS.md

## Build & Run Commands

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo run -- -s localhost -p 1433 -d master -u sa -P 'password'   # Run with SQL Server auth
cargo run -- -s 10.0.0.1 -d MyDb -u user -P 'pass' --trust-cert  # Remote server, trust cert
cargo test                   # Run tests
cargo fmt                    # Format
cargo clippy                 # Lint
```

You need a reachable SQL Server instance to run the app. No file-picker mode — connection is via CLI args only.

Release builds are cross-compiled via CI (`.github/workflows/release.yml`) for macOS (aarch64 + x86_64), Linux (x86_64), and Windows (x86_64). Releases trigger on `v*` tags.

## Architecture

An async **ratatui**-based TUI SQL Server browser built on **tiberius** + **tokio**:

```
main.rs          Entry point: CLI parsing (clap), async tokio runtime, terminal setup, event loop
  ├── app.rs     Data layer: async tiberius connections, domain types (App, Db, Table, CellType, ConnectionConfig)
  └── ui.rs      UI shell: owns Ui struct, delegates to sub-modules
        └── ui/
            ├── talbe_view.rs   Main database view (nav sidebar + data table + query editor)
            ├── help_view.rs    "?" help overlay
            ├── string_list.rs  Reusable selectable list widget (StringList)
            ├── colors.rs       Centralized color/style constants
            └── utils.rs        Layout utility (center)
```

### Control Flow

1. `main.rs` runs an **async** event loop inside `#[tokio::main]`: `terminal.draw()` → `ui.ui()`, then `event::read()` → `ui.handle_input().await`
2. `App::connect()` is called once at startup, then all query methods (`select`, `execute_sql`) open a fresh tiberius `Client` per call via `get_client()`.
3. `TableView` manages sub-state: navigation tab (Tables/Views), content tab (Browse/Schema/Query), search mode, copy menu, SQL autocomplete.

### Data Flow

- `App::connect()` queries `INFORMATION_SCHEMA.TABLES` to discover tables/views, then fetches row counts and column names from `INFORMATION_SCHEMA.COLUMNS`.
- `App::select()` / `App::execute_sql()` return `(Vec<String>, Vec<Vec<CellType>>)` — column names and rows. SQL values are mapped to `CellType` (Text/Null/Blob).
- Table names are bracket-quoted as `[schema].[name]` in generated SQL to handle special characters and reserved words.
- Schema info includes a `schema` field (e.g., `"dbo"`, `"sales"`) — table identity is `schema.name`.

## Key Differences from sqltui

| Aspect | sqltui | mssqltui |
|--------|--------|----------|
| Database | SQLite (local file) | SQL Server (network) |
| Runtime | Synchronous | Async (tokio) |
| Connection | Opens file via file picker | CLI args (`-s`, `-p`, `-d`, `-u`, `-P`) |
| Schema query | `sqlite_master` + `PRAGMA table_info` | `INFORMATION_SCHEMA.TABLES` / `.COLUMNS` |
| Identifier quoting | Double quotes (`"name"`) | Brackets (`[name]`) |
| Table identity | `name` | `schema.name` |
| File picker | Yes (`.db`/`.sqlite3`/`.db3`) | No |
| SQL dialect keywords | SQLite | T-SQL (TOP, NOLOCK, FETCH, etc.) |
| Cell type mapping | `rusqlite::ValueRef` | Sequential `try_get` attempts per type |

## Key Conventions

- **Module naming**: `talbe_view.rs` is a deliberate misspelling preserved from the original sqltui codebase.
- **Vim-like keybindings**: `i`=up, `k`=down, `j`=left, `l`=right in the data table. Uppercase `J`/`K` for list navigation.
- **Input modes**: Mutually exclusive (search, query, copy menu, normal) that intercept key events at different levels.
- **Colors**: All color constants in `ui/colors.rs`. Import via `use super::{PRIMARY_COLOR, ...}`.
- **Error handling**: `color_eyre` for panic reports. Functions return `Result<_, Box<dyn std::error::Error>>`.
- **Async pattern**: `handle_input` and `load_table_data` are `async fn` — the event loop `await`s them.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` + `crossterm` | TUI framework + terminal backend |
| `tiberius` | SQL Server TDS protocol client |
| `tokio` | Async runtime |
| `tokio-util` (compat) | `Compat<WriteHalf>` adapter for tiberius over tokio TcpStream |
| `futures-util` | `TryStreamExt` for tiberius `QueryStream` iteration |
| `clap` (derive) | CLI argument parsing |
| `color-eyre` | Error reporting / panic hooks |
| `arboard` | System clipboard access |
| `strum` | Enum iteration/display for tab types |

## Gotchas

- **No test suite yet.** `cargo test` compiles but finds nothing.
- **Connection-per-query pattern**: `App` doesn't hold a persistent connection. Each `select()`/`execute_sql()` creates a new TCP connection + tiberius client. This avoids connection state issues but has overhead.
- **Cell indexing**: `TableState` column 0 is the row number (`#`). Data columns start at index 1. Use `col.saturating_sub(1)` for data column index.
- **tiberius `try_get`**: Types are not `Option<T>` — use `try_get::<&str, _>` (borrowed) not `try_get::<Option<&str>, _>`. The return is `Result<Option<T>>`.
- **`AuthMethod`**: `SqlServerAuth` is `pub(crate)` — always use `AuthMethod::sql_server(user, password)`. There is no Windows integrated auth on non-Windows platforms.
- **Key release filtering**: The event loop skips `KeyEventKind::Release` events (macOS sends both press and release). Ctrl+Q is checked separately from the main input dispatch.
- **`dbo` prefix hiding**: Tables in the `dbo` schema display without the prefix in the nav list for cleaner appearance, but the internal key is always `schema.name`.
