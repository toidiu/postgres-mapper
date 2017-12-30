#[macro_use] extern crate postgres_mapper_derive;

extern crate postgres;
extern crate postgres_mapper;

use postgres::{Connection, TlsMode};
use std::error::Error;

#[derive(Debug, PostgresMapper)]
pub struct User {
    id: i32,
    name: String,
    email: Option<String>,
}

fn try_main() -> Result<(), Box<Error>> {
    let conn = Connection::connect(
        "postgresql://postgres@127.0.0.1:5432",
        TlsMode::None,
    )?;

    conn.execute("create table if not exists users (
        id serial primary key,
        name text not null,
        email text
    )", &[])?;

    conn.execute(
        "insert into users (name, email) values ($1, $2)",
        &[&"test", &None::<String>],
    )?;

    conn.query("select id, name, email from users", &[])?
        .iter()
        .map(User::from)
        .for_each(|user| {
            println!("User ID {} has a name of {}", user.id, user.name);
        });

    conn.execute("drop table users", &[])?;

    Ok(())
}

fn main() {
    try_main().unwrap();
}
