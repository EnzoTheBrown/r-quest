use self::model::{NewVariable, Variable};
use model;
use schema;

fn create_variable(
    conn: &mut diesel::SqliteConnection,
    new_variable: NewVariable,
) -> Result<Variable, diesel::result::Error> {
    use schema::variable;

    diesel::insert_into(variable::table)
        .values(&new_variable)
        .execute(conn)?;

    variable::table
        .filter(variable::id.eq(new_variable.id))
        .first(conn)
}

fn get_variable_by_id(
    conn: &mut diesel::SqliteConnection,
    variable_id: i32,
) -> Result<Variable, diesel::result::Error> {
    use schema::variable::dsl::*;

    variable.filter(id.eq(variable_id)).first(conn)
}

fn list_variables(
    conn: &mut diesel::SqliteConnection,
) -> Result<Vec<Variable>, diesel::result::Error> {
    use schema::variable::dsl::*;

    variable.load(conn)
}
