use crate::schema::{project, request, variable};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = project)]
pub struct NewProject {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub endpoint: String,
    pub project_id: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = request)]
pub struct NewRequest {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub endpoint: String,
    pub project_id: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Variable {
    pub id: Option<i32>,
    pub name: String,
    pub value: Option<String>,
    pub project_id: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = variable)]
pub struct NewVariable {
    pub id: i32,
    pub name: String,
    pub value: Option<String>,
    pub project_id: i32,
    pub created_at: chrono::NaiveDateTime,
}
