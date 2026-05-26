# mssqltui

[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub release](https://img.shields.io/github/v/release/dzulfikar08/mssqltui?include_prereleases)](https://github.com/dzulfikar08/mssqltui/releases)

A blazing fast TUI SQL Server (MSSQL) browser built with Rust. Browse tables, inspect schemas, run T-SQL queries with syntax highlighting and autocomplete — all from your terminal. Includes a built-in Web UI.

**[Releases](https://github.com/dzulfikar08/mssqltui/releases)** | **[Contributing](CONTRIBUTING.md)**

## Features

- **Browse tables & views** — Navigate your database with a sidebar showing schema-qualified table names (dbo.Orders, sales.Invoices), live row counts, and instant data preview
- **Schema viewer** — Inspect table structures with column names and types from INFORMATION_SCHEMA
- **T-SQL query editor** — Write and execute SQL with syntax highlighting (TOP, NOLOCK, EXEC, and other T-SQL keywords) and autocomplete
- **Multi-statement scripts** — Run multiple SQL statements at once. Results shown in separate tabs with per-statement error reporting
- **Web UI included** — Launch with `--ui` for a full dark-theme web interface. Same features — browsing, querying, exporting
- **Autocomplete** — Suggests SQL keywords, table names, and `table.column` pairs as you type
- **Copy menu** — Copy cell value, row, row with headers, all results, CREATE statement, or the query itself
- **Search/filter** — Quickly filter the table list with `/`
- **Modern dark theme** — Styled nulls, row numbers, alternating row colors, scrollbar
- **Resilient connections** — Auto-retry on connection failure. The Web UI reconnects on each API call
- **Multi-format export** — Export tables as CSV, JSON, or SQL INSERT statements with proper [schema].[name] quoting

## Screenshot

![mssqltui](assets/screenshot.png)

## Installation

### From Release

Download the latest binary for your platform from the [Releases](https://github.com/dzulfikar08/mssqltui/releases) page.

```bash
mkdir -p ~/.local/bin
cp ~/Downloads/mssqltui-aarch64-macos ~/.local/bin/mssqltui
chmod +x ~/.local/bin/mssqltui
```

Then add `~/.local/bin` to your PATH (if not already) in `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### From Source (cargo)

```bash
cargo install --git https://github.com/dzulfikar08/mssqltui
```

### From Source (manual)

Requirements:
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (for building the Web UI)

```bash
git clone https://github.com/dzulfikar08/mssqltui.git
cd mssqltui

# Build web frontend (optional — only needed for --ui mode)
cd web && npm install && npm run build && cd ..

cargo build --release
cp target/release/mssqltui /usr/local/bin/
```

## Usage

Connect with SQL Server authentication:

```bash
mssqltui -s localhost -p 1433 -d MyDatabase -u sa -P 'YourPassword'
```

Launch with Web UI:

```bash
mssqltui -s localhost -d master -u sa -P 'YourPass123!' --ui
# Opens browser at http://localhost:5000
```

Connect to a remote server with certificate trust:

```bash
mssqltui -s 192.168.1.100 -d Production -u admin -P 'password' --trust-cert
```

### CLI Options

| Flag | Default | Description |
|------|---------|-------------|
| `-s, --server` | `localhost` | SQL Server hostname or IP |
| `-p, --port` | `1433` | SQL Server port |
| `-d, --database` | `master` | Database name |
| `-u, --user` | _(none)_ | Username (SQL Server auth) |
| `-P, --pass` | _(none)_ | Password |
| `--trust-cert` | `false` | Trust server certificate |
| `--no-encrypt` | `false` | Disable TLS encryption |
| `--ui` | `false` | Launch Web UI on port 5000 |
| `--listen` | `5000` | Web UI listen port |
| `--help` | | Show help |
| `--version` | | Show version |

### Keybindings

#### Navigation (Left Sidebar)

| Key | Action |
|-----|--------|
| `q` / `e` | Switch Tables / Views |
| `↑` / `↓` | Navigate list |
| `/` | Search filter |

#### Table View

| Key | Action |
|-----|--------|
| `H` / `L` / `;` | Schema / Browse / Query tab |
| `i` / `k` | Move up / down |
| `j` / `l` | Move left / right |
| `↑` / `↓` / `←` / `→` | Arrow key navigation |
| `u` / `d` | Page up / down half |
| `y` | Yank cell to clipboard |
| `c` | Copy menu |
| `?` | Help |
| `Esc` | Back to Browse tab |
| `Ctrl+Q` | Quit |

#### SQL Query Tab

| Key | Action |
|-----|--------|
| `Ctrl+S` | Run query |
| `[` / `]` | Cycle through multi-statement results |
| `Tab` / `Enter` | Accept autocomplete suggestion |
| `↑` / `↓` | Navigate suggestions |
| `Ctrl+U` | Clear input |
| `Ctrl+W` | Delete last word |
| `Esc` | Clear and return to Browse |

## Web UI

Launch with `--ui` to start a local web server:

```bash
mssqltui -s localhost -d MyDatabase -u sa -P 'password' --ui
```

The Web UI provides the same features as the TUI — browse tables, run T-SQL queries with syntax highlighting and autocomplete, export data as CSV/JSON/SQL. Perfect for demos, screen sharing, and team collaboration.

## Script Execution

Run multiple SQL statements at once. Each statement's result appears in its own tab:

```sql
SELECT TOP 5 * FROM Orders;
SELECT COUNT(*) AS total FROM Customers;
SELECT * FROM Products WHERE price > 100;
```

Errors in one statement won't abort other statements — each result shows success or failure independently.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

## Acknowledgments

Built with [Ratatui](https://ratatui.rs/) and [Tiberius](https://github.com/prisma/tiberius).

## Sponsor

If you find mssqltui useful, consider supporting development:

[Donate via Saweria](https://saweria.co/dzulfikar08)

## Sponsor

This project is sponsored by [Voltrus.id](https://voltrus.id) and [JoyoDigitama.com](https://joyodigitama.com).
