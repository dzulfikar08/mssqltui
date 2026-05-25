use serde::Serialize;
use tiberius::{AuthMethod, Client, Config, QueryItem, Row};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use futures_util::TryStreamExt;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CellType {
    Text(String),
    Null,
    Blob,
}

impl CellType {
    pub fn display_text(&self) -> &str {
        match self {
            CellType::Text(s) => s,
            CellType::Null => "null",
            CellType::Blob => "[Blob]",
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Db {
    pub server: String,
    pub database: String,
    pub tables: Vec<Table>,
    pub views: Vec<Table>,
}

#[derive(Debug, Default, Serialize)]
pub struct Table {
    pub name: String,
    pub schema: String,
    pub sql: String,
    pub row_count: Option<usize>,
    pub columns: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct App {
    pub current_db: Option<Db>,
    pub config: Option<ConnectionConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionConfig {
    pub server: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub trust_cert: bool,
    pub encrypt: bool,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<CellType>>,
    pub rows_affected: usize,
}

impl App {
    pub async fn connect(&mut self, config: ConnectionConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = self.build_client(&config).await?;

        let (mut tables, mut views) = get_tables(&mut client).await?;

        for table in &mut tables {
            table.row_count = Some(count_rows(&mut client, &table.schema, &table.name).await);
            table.columns = get_columns(&mut client, &table.schema, &table.name).await;
        }
        for view in &mut views {
            view.row_count = Some(count_rows(&mut client, &view.schema, &view.name).await);
            view.columns = get_columns(&mut client, &view.schema, &view.name).await;
        }

        self.config = Some(config.clone());
        self.current_db = Some(Db {
            server: config.server,
            database: config.database,
            tables,
            views,
        });
        Ok(())
    }

    async fn build_client(
        &self,
        config: &ConnectionConfig,
    ) -> Result<Client<tokio_util::compat::Compat<tokio::net::TcpStream>>, Box<dyn std::error::Error>> {
        let mut tconfig = Config::new();
        tconfig.host(&config.server);
        tconfig.port(config.port);
        tconfig.database(&config.database);

        tconfig.authentication(AuthMethod::sql_server(
            if config.username.is_empty() { "" } else { &config.username },
            &config.password,
        ));

        if !config.encrypt {
            tconfig.encryption(tiberius::EncryptionLevel::NotSupported);
        }

        if config.trust_cert {
            tconfig.trust_cert();
        }

        let tcp = TcpStream::connect(format!("{}:{}", config.server, config.port)).await?;
        let client = Client::connect(tconfig, tcp.compat_write()).await?;
        Ok(client)
    }

    async fn get_client(
        &self,
    ) -> Result<Client<tokio_util::compat::Compat<tokio::net::TcpStream>>, Box<dyn std::error::Error>> {
        let config = self.config.as_ref().ok_or("Not connected")?;
        self.build_client(config).await
    }

    pub async fn select(
        &self,
        table: &Table,
    ) -> Result<(Vec<String>, Vec<Vec<CellType>>), Box<dyn std::error::Error>> {
        let mut client = self.get_client().await?;
        let quoted = format!("[{}].[{}]", table.schema, table.name);
        let sql = format!("SELECT * FROM {}", quoted);
        let mut stream = client.query(&sql, &[]).await?;

        let cols: Vec<String> = stream
            .columns()
            .await
            .map(|opt| {
                opt.map(|cs| cs.iter().map(|c| c.name().to_string()).collect())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let mut data: Vec<Vec<CellType>> = Vec::new();
        while let Some(item) = stream.try_next().await? {
            match item {
                QueryItem::Row(row) => {
                    let mut cells = Vec::new();
                    for i in 0..cols.len() {
                        cells.push(map_cell(&row, i));
                    }
                    data.push(cells);
                }
                QueryItem::Metadata(_) => {}
            }
        }
        Ok((cols, data))
    }

    pub async fn select_page(
        &self,
        table: &Table,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<String>, Vec<Vec<CellType>>), Box<dyn std::error::Error>> {
        let mut client = self.get_client().await?;
        let quoted = format!("[{}].[{}]", table.schema, table.name);
        let sql = format!(
            "SELECT * FROM {} ORDER BY (SELECT NULL) OFFSET {} ROWS FETCH NEXT {} ROWS ONLY;",
            quoted, offset, limit
        );
        let mut stream = client.query(&sql, &[]).await?;

        let cols: Vec<String> = stream
            .columns()
            .await
            .map(|opt| {
                opt.map(|cs| cs.iter().map(|c| c.name().to_string()).collect())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let mut data: Vec<Vec<CellType>> = Vec::new();
        while let Some(item) = stream.try_next().await? {
            match item {
                QueryItem::Row(row) => {
                    let mut cells = Vec::new();
                    for i in 0..cols.len() {
                        cells.push(map_cell(&row, i));
                    }
                    data.push(cells);
                }
                QueryItem::Metadata(_) => {}
            }
        }
        Ok((cols, data))
    }

    pub async fn execute_sql(
        &self,
        sql: &str,
    ) -> Result<Vec<QueryResult>, Box<dyn std::error::Error>> {
        let sql_trimmed = sql.trim();
        if sql_trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let stmts = split_sql(sql_trimmed);
        if stmts.is_empty() {
            return Ok(Vec::new());
        }
        let mut client = self.get_client().await?;
        let mut results = Vec::new();
        for (_raw, trimmed) in &stmts {
            let result = exec_statement(&mut client, trimmed).await?;
            results.push(result);
        }
        Ok(results)
    }

    pub async fn execute_multi(
        &self,
        sql: &str,
    ) -> Vec<Result<QueryResult, Box<dyn std::error::Error>>> {
        let sql_trimmed = sql.trim();
        if sql_trimmed.is_empty() {
            return Vec::new();
        }
        let stmts = split_sql(sql_trimmed);
        if stmts.is_empty() {
            return Vec::new();
        }
        let mut client = match self.get_client().await {
            Ok(c) => c,
            Err(e) => return vec![Err(e)],
        };
        let mut results = Vec::new();
        for (_raw, trimmed) in &stmts {
            let result = exec_statement(&mut client, trimmed).await;
            results.push(result);
        }
        results
    }
}

pub fn split_sql(sql: &str) -> Vec<(String, String)> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == 'N' && i + 1 < chars.len() && chars[i + 1] == '\'' {
            current.push_str("N'");
            i += 2;
            while i < chars.len() {
                current.push(chars[i]);
                if chars[i] == '\'' {
                    if i + 1 < chars.len() && chars[i + 1] == '\'' {
                        i += 1;
                        current.push(chars[i]);
                    } else {
                        break;
                    }
                }
                i += 1;
            }
        } else if chars[i] == '\'' {
            current.push(chars[i]);
            i += 1;
            while i < chars.len() {
                current.push(chars[i]);
                if chars[i] == '\'' {
                    if i + 1 < chars.len() && chars[i + 1] == '\'' {
                        i += 1;
                        current.push(chars[i]);
                    } else {
                        break;
                    }
                }
                i += 1;
            }
        } else if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
            current.push_str("--");
            i += 2;
            while i < chars.len() && chars[i] != '\n' {
                current.push(chars[i]);
                i += 1;
            }
        } else if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            current.push_str("/*");
            i += 2;
            while i + 1 < chars.len() {
                current.push(chars[i]);
                if chars[i] == '*' && chars[i + 1] == '/' {
                    current.push('/');
                    i += 2;
                    break;
                }
                i += 1;
            }
        } else if chars[i] == '[' {
            current.push('[');
            i += 1;
            while i < chars.len() && chars[i] != ']' {
                current.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                current.push(']');
            }
        } else if chars[i] == '"' {
            current.push(chars[i]);
            i += 1;
            while i < chars.len() {
                current.push(chars[i]);
                if chars[i] == '"' {
                    if i + 1 < chars.len() && chars[i + 1] == '"' {
                        i += 1;
                        current.push(chars[i]);
                    } else {
                        break;
                    }
                }
                i += 1;
            }
        } else if chars[i] == ';' {
            current.push(';');
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() && trimmed != ";" {
                statements.push((current.clone(), trimmed));
            }
            current.clear();
        } else if chars[i] == 'G' || chars[i] == 'g' {
            let is_go =
                (i == 0 || chars[i - 1] == '\n' || chars[i - 1] == ';') &&
                (i + 1 < chars.len() && (chars[i + 1] == 'o' || chars[i + 1] == 'O')) &&
                (i + 2 >= chars.len() || chars[i + 2] == '\n' || chars[i + 2] == ';' || chars[i + 2] == '\r');
            if is_go {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    statements.push((current.clone(), trimmed));
                }
                current.clear();
                while i < chars.len() && chars[i] != '\n' && chars[i] != ';' {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '\n' { current.push('\n'); }
                continue;
            } else {
                current.push(chars[i]);
            }
        } else {
            current.push(chars[i]);
        }
        i += 1;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        statements.push((current, trimmed));
    }
    statements
}

pub async fn exec_statement(
    client: &mut Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    sql: &str,
) -> Result<QueryResult, Box<dyn std::error::Error>> {
    let sql = sql.trim();
    if sql.is_empty() {
        return Ok(QueryResult { columns: vec![], rows: vec![], rows_affected: 0 });
    }

    let mut stream = client.query(sql, &[]).await?;

    let cols: Vec<String> = stream
        .columns()
        .await
        .map(|opt| {
            opt.map(|cs| cs.iter().map(|c| c.name().to_string()).collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let mut data: Vec<Vec<CellType>> = Vec::new();
    while let Some(item) = stream.try_next().await? {
        match item {
            QueryItem::Row(row) => {
                let mut cells = Vec::new();
                for i in 0..cols.len() {
                    cells.push(map_cell(&row, i));
                }
                data.push(cells);
            }
            QueryItem::Metadata(_) => {}
        }
    }
    let rows_affected = data.len();
    Ok(QueryResult { columns: cols, rows: data, rows_affected })
}

fn map_cell(row: &Row, col_index: usize) -> CellType {
    if let Ok(Some(s)) = row.try_get::<&str, _>(col_index) {
        return CellType::Text(s.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<i32, _>(col_index) {
        return CellType::Text(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<i64, _>(col_index) {
        return CellType::Text(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<f64, _>(col_index) {
        return CellType::Text(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<f32, _>(col_index) {
        return CellType::Text(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<bool, _>(col_index) {
        return CellType::Text(v.to_string());
    }
    if let Ok(Some(_)) = row.try_get::<&[u8], _>(col_index) {
        return CellType::Blob;
    }
    CellType::Null
}

async fn count_rows(
    client: &mut Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schema: &str,
    table_name: &str,
) -> usize {
    let quoted = format!("[{}].[{}]", schema, table_name);
    let sql = format!("SELECT COUNT(*) FROM {}", quoted);
    let result = async {
        let mut stream = client.query(&sql, &[]).await.ok()?;
        while let Some(item) = stream.try_next().await.ok()? {
            if let QueryItem::Row(r) = item {
                return Some(
                    r.try_get::<i32, _>(0)
                        .ok()
                        .flatten()
                        .map(|v| v as i64)
                        .or_else(|| r.try_get::<i64, _>(0).ok().flatten())
                        .unwrap_or(0),
                );
            }
        }
        None::<i64>
    }
    .await;
    result.unwrap_or(0) as usize
}

async fn get_columns(
    client: &mut Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schema: &str,
    table_name: &str,
) -> Vec<String> {
    let sql = format!(
        "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA='{}' AND TABLE_NAME='{}' ORDER BY ORDINAL_POSITION;",
        schema, table_name
    );
    let mut stream = match client.query(&sql, &[]).await {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let mut cols: Vec<String> = Vec::new();
    loop {
        match stream.try_next().await {
            Ok(Some(QueryItem::Row(row))) => {
                if let Some(name) = row.get::<&str, _>(0) {
                    cols.push(name.to_string());
                }
            }
            Ok(Some(QueryItem::Metadata(_))) => continue,
            Ok(None) => break,
            Err(_) => break,
        }
    }
    cols
}

async fn get_tables(
    client: &mut Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
) -> Result<(Vec<Table>, Vec<Table>), Box<dyn std::error::Error>> {
    let sql = r#"
        SELECT
            t.TABLE_TYPE,
            t.TABLE_SCHEMA,
            t.TABLE_NAME
        FROM INFORMATION_SCHEMA.TABLES t
        WHERE t.TABLE_TYPE IN ('BASE TABLE', 'VIEW')
        ORDER BY t.TABLE_SCHEMA, t.TABLE_NAME;
    "#;
    let mut stream = client.query(sql, &[]).await?;
    let mut tables: Vec<Table> = Vec::new();
    let mut views: Vec<Table> = Vec::new();

    loop {
        match stream.try_next().await? {
            Some(QueryItem::Row(row)) => {
                let type_id: &str = row.get(0).unwrap_or("");
                let schema_name: &str = row.get(1).unwrap_or("dbo");
                let name: &str = row.get(2).unwrap_or("");

                let table = Table {
                    name: name.to_string(),
                    schema: schema_name.to_string(),
                    sql: String::new(),
                    row_count: None,
                    columns: Vec::new(),
                };

                if type_id == "BASE TABLE" {
                    tables.push(table);
                } else if type_id == "VIEW" {
                    views.push(table);
                }
            }
            Some(QueryItem::Metadata(_)) => continue,
            None => break,
        }
    }
    Ok((tables, views))
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
