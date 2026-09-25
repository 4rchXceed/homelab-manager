use turso::{Builder, Connection, Database};

use crate::{database::turso::error::TursoDbError, sql};

pub struct TursoDbConnection {
    database: Database,
}

impl TursoDbConnection {
    pub async fn new(path: String) -> Result<Self, TursoDbError> {
        let db = Builder::new_local(path.as_str())
            .experimental_multiprocess_wal(true)
            .build()
            .await
            .map_err(|e| TursoDbError::ConnectionError(e))?;

        Self::ensure_database(&db).await?;

        return Ok(Self { database: db });
    }

    pub async fn ensure_database(db: &Database) -> Result<(), TursoDbError> {
        let conn = db.connect().map_err(|e| TursoDbError::ConnectionError(e))?;

        let mut table_number = conn
            .query(
                "SELECT COUNT(name) FROM sqlite_master WHERE type='table';",
                (),
            )
            .await
            .map_err(|e| TursoDbError::ExecuteError(e))?;

        let row = table_number
            .next()
            .await
            .map_err(|e| TursoDbError::FetchError(e))?
            .ok_or(TursoDbError::FailedToGetNumberOfTables)?;

        let count: i64 = row.get(0).map_err(|e| TursoDbError::FetchError(e))?;

        if count > 0 {
            return Ok(());
        }

        conn.execute_batch(sql!(init))
            .await
            .map_err(|e| TursoDbError::ExecuteError(e))?;

        return Ok(());
    }

    pub fn get_database(&self) -> &Database {
        return &self.database;
    }

    pub fn create_connection(&self) -> Result<Connection, TursoDbError> {
        let conn = self
            .database
            .connect()
            .map_err(|e| TursoDbError::ConnectionError(e))?;

        return Ok(conn);
    }
}
