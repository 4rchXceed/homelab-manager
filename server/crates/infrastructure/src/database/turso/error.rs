use thiserror::Error;

#[derive(Debug, Error)]
pub enum TursoDbError {
    #[error("Failed to connect to the database: {0}")]
    ConnectionError(turso::Error),
    #[error("Failed to execute the query: {0}")]
    ExecuteError(turso::Error),
    #[error("Failed to fetch the result: {0}")]
    FetchError(turso::Error),
    #[error("Failed to get the number of tables (scalar empty)")]
    FailedToGetNumberOfTables,
}
