#[cfg(feature = "postgres-support")]
extern crate postgres;
#[cfg(feature = "tokio-postgres-support")]
extern crate tokio_postgres;

use postgres::Error as PostgresError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[cfg(feature = "postgres-support")]
use postgres::rows::Row as PostgresRow;
#[cfg(feature = "tokio-postgres-support")]
use tokio_postgres::rows::Row as TokioRow;

/// Trait containing various methods for converting from a postgres Row to a
/// mapped type.
///
/// When using the `postgres_mapper_derive` crate's `PostgresMapper` proc-macro,
/// this will automatically be implemented on types.
///
/// The [`from_postgres_row`] method exists for consuming a `Row` - useful for
/// iterator mapping - while [`from_postgres_row_ref`] exists for borrowing a
/// `Row`.
#[cfg(feature = "postgres-support")]
pub trait FromPostgresRow: Sized {
    /// Converts from a postgres `Row` into a mapped type, consuming the given
    /// `Row`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ColumnNotFound`] if the column in a mapping was not
    /// found.
    ///
    /// Returns [`Error::Postgres`] if there was an error converting the row
    /// column to the requested type.
    ///
    /// [`Error::ColumnNotFound`]: enum.Error.html#variant.ColumnNotFound
    /// [`Error::Postgres`]: enum.Error.html#variant.Postgres
    fn from_postgres_row(row: PostgresRow) -> Result<Self, Error>;

    /// Converts from a `postgres` `Row` into a mapped type, borrowing the given
    /// `Row`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ColumnNotFound`] if the column in a mapping was not
    /// found.
    ///
    /// Returns [`Error::Postgres`] if there was an error converting the row
    /// column to the requested type.
    ///
    /// [`Error::ColumnNotFound`]: enum.Error.html#variant.ColumnNotFound
    /// [`Error::Postgres`]: enum.Error.html#variant.Postgres
    fn from_postgres_row_ref(row: &PostgresRow) -> Result<Self, Error>;
}

/// Trait containing various methods for converting from a `tokio-postgres` Row
/// to a mapped type.
///
/// When using the `postgres_mapper_derive` crate's `PostgresMapper` proc-macro,
/// this will automatically be implemented on types.
///
/// The [`from_tokio_postgres_row`] method exists for consuming a `Row` - useful
/// for iterator mapping - while [`from_postgres_row_ref`] exists for borrowing
/// a `Row`.
#[cfg(feature = "tokio-postgres-support")]
pub trait FromTokioPostgresRow: Sized {
    /// Converts from a `tokio-postgres` `Row` into a mapped type, consuming the
    /// given `Row`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ColumnNotFound`] if the column in a mapping was not
    /// found.
    ///
    /// Returns [`Error::Conversion`] if there was an error converting the row
    /// column to the requested type.
    ///
    /// [`Error::ColumnNotFound`]: enum.Error.html#variant.ColumnNotFound
    /// [`Error::Conversion`]: enum.Error.html#variant.Conversion
    fn from_tokio_postgres_row(row: TokioRow) -> Result<Self, Error>;

    /// Converts from a `tokio-postgres` `Row` into a mapped type, borrowing the
    /// given `Row`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ColumnNotFound`] if the column in a mapping was not
    /// found.
    ///
    /// Returns [`Error::Conversion`] if there was an error converting the row
    /// column into the requested type.
    ///
    /// [`Error::ColumnNotFound`]: enum.Error.html#variant.ColumnNotFound
    /// [`Error::Conversion`]: enum.Error.html#variant.Conversion
    fn from_tokio_postgres_row_ref(row: &TokioRow) -> Result<Self, Error>;
}

/// General error type returned throughout the library.
#[derive(Debug)]
pub enum Error {
    /// A column in a row was not found.
    ColumnNotFound,
    /// An error from the `tokio-postgres` crate while converting a type.
    #[cfg(feature = "tokio-postgres-support")]
    Conversion(Box<StdError + Send + Sync>),
    /// An error from the `postgres` crate while converting a type.
    #[cfg(feature = "postgres-support")]
    Postgres(PostgresError),
}

#[cfg(feature = "tokio-postgres-support")]
impl From<Box<StdError + Send + Sync>> for Error {
    fn from(err: Box<StdError + Send + Sync>) -> Self {
        Error::Conversion(err)
    }
}

#[cfg(feature = "postgres-support")]
impl From<PostgresError> for Error {
    fn from(err: PostgresError) -> Self {
        Error::Postgres(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ColumnNotFound => "Column in row not found",
            #[cfg(feature = "tokio-postgres-support")]
            Error::Conversion(ref inner) => inner.description(),
            #[cfg(feature = "postgres-support")]
            Error::Postgres(ref inner) => inner.description(),
        }
    }
}
