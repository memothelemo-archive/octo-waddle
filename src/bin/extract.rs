use nce_qualifiers::parser::Qualifiers;
use std::fs::File;

fn main() {
    let file = File::open("qualifiers.txt").unwrap();

    let conn = sqlite::open("qualifiers.db").unwrap();
    nce_qualifiers::db::prepare(&conn).expect("failed to prepare database");

    for qualifier in Qualifiers::from_reader(file) {
        let qualifier = qualifier.expect("failed to parse qualifier");
        nce_qualifiers::db::insert_qualifier(&conn, &qualifier)
            .expect("failed to insert qualifier");
    }
}
