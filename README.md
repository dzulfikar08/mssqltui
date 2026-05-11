# mssqltui

A blazing fast TUI SQL Server (MSSQL) browser — browse, query, and manage databases from your terminal.

Built with [Ratatui](https://ratatui.rs/) and [Tiberius](https://github.com/prisma/tiberius).

## Installation

```bash
cargo install --path .
```

## Usage

Connect with SQL Server authentication:

```bash
mssqltui -s localhost -p 1433 -d MyDatabase -u sa -P 'YourPassword'
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

## Key Bindings

### File Navigation

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate file list |
| `Enter` | Open database |

### Table View

| Key | Action |
|-----|--------|
| `H` / `L` / `;` | Switch tabs: Schema / Browse / Query |
| `q` / `e` | Switch: Tables / Views |
| `i`/`j`/`k`/`l` or `↑↓←→` | Move cell |
| `u` / `d` | Page up/down half |
| `y` | Yank cell to clipboard |
| `c` | Copy menu |
| `/` | Search/filter table list |
| `Esc` | Back to Browse tab |
| `Ctrl+S` | Run SQL query (in Query tab) |
| `Tab` / `Enter` | Accept autocomplete suggestion |
| `?` | Help |
| `Ctrl+Q` | Quit |

## Building

```bash
cargo build --release
```

## License

GPL-3.0
