use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};


#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub id: Option<usize>,
    
}

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptionsAll {
    pub id: Option<usize>,
    pub fname: Option<String>,
    pub lname: Option<String>,
    pub username : Option<String>,
    pub userpassword: Option<String>
    
}

#[derive(Deserialize, Debug)]
pub struct UpdateUsersSchema {
    pub fname: Option<String>,
    pub lname: Option<String>,
    pub username: Option<String>,
    pub userpassword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserWithTodosSchema {
    pub fname: String,
    pub lname: String,
    pub username: String,
    pub userpassword: String,
    pub todos: Vec<CreateTodoSchema>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUsersSchema {
    pub fname: String,
    pub lname: String,
    pub username: String,
    pub userpassword: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodoSchema {
    pub todoname: String,
}

#[derive(Serialize, Debug)]
pub struct TodoModelResponse {
    pub todoname: String,
    
}

#[derive(Debug, Deserialize,Serialize)]
pub struct DateRangeQuery {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTodoSchema {
    pub todoid: u64,
    pub todoname: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteTodoSchema {
    pub user_id: u64,
    pub todoid: u64,
}