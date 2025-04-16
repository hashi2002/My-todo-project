use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use serde_json::json;

use crate::{
    model::{UsersModel, UsersModelResponse, TodosModel},
    schema::{CreateUsersSchema,  UpdateUsersSchema,UpdateTodoSchema,DateRangeQuery,{CreateTodoSchema}},
    AppState,
};


pub async fn todoapp_handler() -> impl IntoResponse {
    const MESSAGE: &str = "API Services";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": MESSAGE
    });

    Json(json_response)
}


pub async fn create_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateUsersSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query(
        r#"INSERT INTO users (fname, lname, username, userpassword) VALUES (?, ?, ?, ?)"#
    )
    .bind(&body.fname)
    .bind(&body.lname)
    .bind(&body.username)
    .bind(&body.userpassword)
    .execute(&data.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "message": format!("User insert error: {:?}", e)}))
        )
    })?;

    let user_id = result.last_insert_id();

    Ok(Json(json!({
        "status": "success",
        "message": "User created successfully",
        "user_id": user_id
    })))
}


pub async fn create_todos_for_user_handler(
    Path(user_id): Path<u64>, 
    State(data): State<Arc<AppState>>,
    Json(todos): Json<Vec<CreateTodoSchema>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": format!("DB error: {:?}", e) }))
        ))?;

    if user_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "fail", "message": "User not found" }))
        ));
    }

    for todo in todos {
        sqlx::query("INSERT INTO todolist (user_id, todoname) VALUES (?, ?)")
            .bind(user_id)
            .bind(&todo.todoname)
            .execute(&data.db)
            .await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": format!("Insert error: {:?}", e) }))
            ))?;
    }

    Ok(Json(json!({
        "status": "success",
        "message": "Todos added successfully",
        "user_id": user_id
    })))
}


pub async fn get_user_with_todos_handler(
    Path(id): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let user = sqlx::query_as::<_, UsersModel>(r#"SELECT * FROM users WHERE id = ?"#)
        .bind(&id)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "message": format!("{:?}", e)}))
        ))?;

    if let Some(user) = user {
        let todos = sqlx::query_as::<_, TodosModel>(r#"SELECT * FROM todolist WHERE user_id = ?"#)
            .bind(&id)
            .fetch_all(&data.db)
            .await
            .unwrap_or_default();

        let response = json!({
            "status": "success",
            "data": {
                "user": to_users_response(&user),
                "todos": todos
            }
        });

        Ok(Json(response))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!({"status": "fail", "message": "User not found"})),
        ))
    }
}


pub async fn list_users_handler(
    State(data): State<Arc<AppState>>, 
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let query_result = sqlx::query_as::<_, UsersModel>("SELECT * FROM users")
        .fetch_all(&data.db)  
        .await;


    match query_result {
        Ok(users) => {
           
            let user_list: Vec<UsersModelResponse> = users
                .into_iter()
                .map(|user| to_users_response(&user))  
                .collect();

            
            let users_response = json!({
                "status": "success",
                "data": {
                    "users": user_list
                }
            });

            Ok((StatusCode::OK, Json(users_response)))  
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,  
            Json(json!({"status": "error", "message": format!("{:?}", e)})),
        )),
    }
}


pub async fn user_todos_by_date_handler(
    Path(user_id): Path<u64>,  
    State(data): State<Arc<AppState>>,  
    Query(query): Query<DateRangeQuery>,  
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

  
    let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": format!("DB error: {:?}", e)})),
            )
        })?;

    if user_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"status": "fail", "message": "User not found"})),
        ));
    }


    let todos = sqlx::query_as::<_, TodosModel>(r#"
        SELECT * 
        FROM todolist 
        WHERE user_id = ? 
        AND created_at BETWEEN ? AND ?
    "#)
    .bind(user_id)
    .bind(&query.start_date)  
    .bind(&query.end_date)    
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "message": format!("Query error: {:?}", e)})),
        )
    })?;

    Ok(Json(json!({
        "status": "success",
        "user_id": user_id,
        "from": query.start_date,
        "to": query.end_date,
        "todos": todos
    })))
}



fn to_users_response(user: &UsersModel) -> UsersModelResponse {
    UsersModelResponse {
        id: user.id,
        fname: user.fname.to_owned(),
        lname: user.lname.to_owned(),
        username: user.username.to_owned(),
        userpassword: user.userpassword.to_owned(),
    }
}

pub async fn delete_user_with_todos_handler(
    Path(id): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut tx = data.db.begin().await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"status": "error", "message": format!("{:?}", e)})),
    ))?;

    sqlx::query("DELETE FROM todolist WHERE user_id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": format!("{:?}", e)}))))?;

    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": format!("{:?}", e)}))))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"message": "User not found"}))));
    }

    tx.commit().await.unwrap();

    Ok(Json(json!({"status": "success", "message": "User and todos deleted"})))
}



pub async fn delete_todo_for_user_handler(
    Path((user_id, todoid)): Path<(u64, u64)>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user exists
    let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": format!("DB error: {:?}", e) }))
            )
        })?;

    if user_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "fail", "message": "User not found" }))
        ));
    }

    let result = sqlx::query("DELETE FROM todolist WHERE todoid = ? AND user_id = ?")
        .bind(todoid)
        .bind(user_id)
        .execute(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": format!("Delete error: {:?}", e) }))
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "fail",
                "message": format!("Todo not found for user_id {} with todoid {}", user_id, todoid)
            }))
        ));
    }

    Ok(Json(json!({
        "status": "success",
        "message": format!("Todo with id {} deleted successfully for user {}", todoid, user_id)
    })))
}

pub async fn update_user_handler(
    Path(id): Path<String>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateUsersSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
   


    if body.fname.is_none()
        && body.lname.is_none()
        && body.username.is_none()
        && body.userpassword.is_none()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "fail",
                "message": "At least one field must be provided for update"
            })),
        ));
    }


    let user = sqlx::query_as::<_, UsersModel>(r#"SELECT * FROM users WHERE id = ?"#)
        .bind(&id)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            eprintln!("DB error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": format!("DB error: {:?}", e)})),
            )
        })?;

    if user.is_none() {
        println!("User with ID {} not found", id);
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"status": "fail", "message": "User not found"})),
        ));
    }

    let mut user = user.unwrap();

    
    if let Some(fname) = &body.fname {
        user.fname = fname.to_owned();
    }
    if let Some(lname) = &body.lname {
        user.lname = lname.to_owned();
    }
    if let Some(username) = &body.username {
        user.username = username.to_owned();
    }
    if let Some(userpassword) = &body.userpassword {
        user.userpassword = userpassword.to_owned();
    }

    let result = sqlx::query(
        r#"
        UPDATE users
        SET fname = ?, lname = ?, username = ?, userpassword = ?
        WHERE id = ?
        "#,
    )
    .bind(&user.fname)
    .bind(&user.lname)
    .bind(&user.username)
    .bind(&user.userpassword)
    .bind(&id)
    .execute(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Update error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "message": format!("Update error: {:?}", e)})),
        )
    })?;

    if result.rows_affected() == 0 {
        println!("No rows affected. User may not exist.");
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"status": "fail", "message": "User not found"})),
        ));
    }

 
    Ok(Json(json!({
        "status": "success",
        "message": "User updated successfully",
        "data": to_users_response(&user)
    })))
}



pub async fn update_todos_for_user_handler(
    Path(user_id): Path<u64>,
    State(data): State<Arc<AppState>>,
    Json(todos): Json<Vec<UpdateTodoSchema>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Check if user exists
    let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": format!("DB error: {:?}", e) }))
        ))?;

    if user_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "fail", "message": "User not found" }))
        ));
    }


    for todo in todos {
        let result = sqlx::query(
            "UPDATE todolist SET todoname = ? WHERE todoid = ? AND user_id = ?"
        )
        .bind(&todo.todoname)
        .bind(todo.todoid)
        .bind(user_id)
        .execute(&data.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": format!("Update error: {:?}", e) }))
        ))?;

        if result.rows_affected() == 0 {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": "fail",
                    "message": format!("Todo with id {} not found for user {}", todo.todoid, user_id)
                }))
            ));
        }
    }

    Ok(Json(json!({
        "status": "success",
        "message": "Todos updated successfully",
        "user_id": user_id
    })))
}