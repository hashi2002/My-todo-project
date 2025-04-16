use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;


#[derive(Debug, Serialize, Deserialize)]
pub struct TodosModelResponse {
    pub todoid: i64,
    pub todoname: String,
    pub created_at: Option<NaiveDateTime>,
}
#[derive(FromRow)]
pub struct UsersModel {
    pub id: i32,
    pub fname: String,
    pub lname: String,
    pub username: String,
    pub userpassword: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct TodosModel {
    pub todoid: i32,
    pub user_id: i32,
    pub todoname: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct UsersModelResponse {
    pub id: i32,
    pub fname: String,
    pub lname: String,
    pub username: String,
    pub userpassword: String,
}

#[derive(Debug,Serialize)]
pub struct TodoModelResponse {
    pub todoid : i32,
    pub todoname: String,
    
}
