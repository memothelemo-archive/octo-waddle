use crate::types::RawQualifier;
use error_stack::{Result, ResultExt};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("failed to operate database")]
pub struct Error;

pub fn insert_qualifier(conn: &sqlite::Connection, qualifier: &RawQualifier) -> Result<(), Error> {
    // We need to insert test center first.
    let mut stmt = conn
        .prepare("SELECT EXISTS(SELECT 1 FROM test_centers WHERE id = ?);")
        .change_context(Error)?;

    stmt.bind((1, qualifier.test_center_code as i64))
        .change_context(Error)?;

    let mut exists = false;
    while let sqlite::State::Row = stmt.next().change_context(Error)? {
        exists = stmt.read::<i64, _>(0).change_context(Error)? == 1;
    }
    drop(stmt);

    if !exists {
        let mut stmt = conn
            .prepare(r#"INSERT INTO test_centers (id, name, address) VALUES (?, ?, ?);"#)
            .change_context(Error)?;

        stmt.bind((1, qualifier.test_center_code as i64))
            .change_context(Error)?;
        stmt.bind((2, qualifier.test_center_name.as_str()))
            .change_context(Error)?;
        stmt.bind((3, qualifier.test_center_addr.as_str()))
            .change_context(Error)?;

        assert_eq!(stmt.next().change_context(Error)?, sqlite::State::Done);
    }

    let mut stmt = conn.prepare(
        r#"
    INSERT OR IGNORE INTO examinees(id, surname, first_name, middle_name, seat_number, time, room_assignment, test_center_code)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?);
    "#,
    ).change_context(Error)?;

    stmt.bind((1, qualifier.id as i64)).change_context(Error)?;
    stmt.bind((2, qualifier.surname.as_str()))
        .change_context(Error)?;
    stmt.bind((3, qualifier.first_name.as_str()))
        .change_context(Error)?;
    stmt.bind((4, qualifier.middle_name.as_deref()))
        .change_context(Error)?;
    stmt.bind((5, qualifier.seat_number as i64))
        .change_context(Error)?;
    stmt.bind((6, qualifier.time.as_str()))
        .change_context(Error)?;
    stmt.bind((7, qualifier.room_assignment as i64))
        .change_context(Error)?;
    stmt.bind((8, qualifier.test_center_code as i64))
        .change_context(Error)?;

    assert_eq!(stmt.next().change_context(Error)?, sqlite::State::Done);

    Ok(())
}

pub fn prepare(conn: &sqlite::Connection) -> Result<(), Error> {
    conn.execute(
        r#"
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS test_centers (
        id          integer PRIMARY KEY,
        name        text NOT NULL,
        address     text NOT NULL
    );

    CREATE TABLE IF NOT EXISTS examinees (
        id                  integer PRIMARY KEY,
        surname             text NOT NULL,
        first_name          text NOT NULL,
        middle_name         text,
        seat_number         integer,
        time                text NOT NULL,
        room_assignment     integer,
        test_center_code    integer,

        FOREIGN KEY (test_center_code)
            REFERENCES test_centers (id)
    );
    "#,
    )
    .change_context(Error)?;

    Ok(())
}
