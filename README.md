# postgres-mapper

`postgres-mapper` is a proc-macro designed to make mapping from postgresql
tables to structs simple.

### Why?

It can be frustrating to write a lot of boilerplate and, ultimately, duplicated
code for mapping from postgres Rows into structs.

For example, this might be what someone would normally write:

```rust
extern crate postgres;

use postgres::rows::Row;

pub struct User {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

impl From<Row> for User {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
        }
    }
}

// code to execute a query here and get back a row
let user = User::from(row); // this can panic
```

This becomes worse when manually implementating using the non-panicking
`get_opt` method variant.

Using this crate, the boilerplate is removed, and panicking and non-panicking
implementations are derived:

```rust
#[macro_use] extern crate postgres_mapper_derive;
extern crate postgres_mapper;

use postgres_mapper::FromPostgresRow;

#[derive(PostgresMapper)]
#[pg_mapper(table = "user")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

// Code to execute a query here and get back a row might now look like:
let stmt = "SELECT {$1} FROM {$2}
    WHERE username = {$3} AND password = {$4}";

let rows = &self
    .conn
    .query(
        stmt,
        &[&User::sql_fields(), &User::sql_table(), username, pass],
    ).unwrap();

let user = rows
    .iter()
    .next()
    .map(|row|
      // `postgres_mapper::FromPostgresRow`'s methods do not panic and return a Result
      User::from_postgres_row(row)?
    );

```

### The two crates

This repository contains two crates: `postgres-mapper` which contains an `Error`
enum and traits for converting from a `postgres` or `tokio-postgres` `Row`
without panicking, and `postgres-mapper-derive` which contains the proc-macro.

`postgres-mapper-derive` has 3 features that can be enabled (where T is the
struct being derived with the provided `PostgresMapper` proc-macro):

- `postgres-support`, which derives
`impl<'a> From<::postgres::rows::Row<'a>> for T` and
`impl<'a> From<&'a ::postgres::Row<'a>> for T` implementations
- `tokio-postgres-support`, which derives
`impl From<::tokio_postgres::rows::Row> for T` and
`impl From<&::tokio_postgres::rows::Row> for T` implementations
- `postgres-mapper` which, for each of the above features, implements
`postgres-mapper`'s `FromPostgresRow` and/or `FromTokioPostgresRow` traits

`postgres-mapper` has two features, `postgres-support` and
`tokio-postgres-support`. When one is enabled in `postgres-mapper-derive`, it
must also be enabled in `postgres-mapper`.

### Installation

The above might be confusing, so here's an example where `tokio-postgres` is
enabled in both crates:

Add the following to your `Cargo.toml`:

```toml
[dependencies.postgres-mapper]
features = ["tokio-postgres-support"]
git = "https://github.com/zeyla/postgres-mapper"

[dependencies.postgres-mapper-derive]
features = ["postgres-mapper", "tokio-postgres-support"]
git = "https://github.com/zeyla/postgres-mapper"
```

This will derive implementations for converting from owned and referenced
`tokio-postgres::rows::Row`s, as well as implementing `postgres-mapper`'s
`FromTokioPostgresRow` trait for non-panicking conversions.

### License

ISC.
