use std::sync::Arc;
use axum::{
    routing::{get, post, delete,put},
    Router,
};

use crate::handler::{
    create_user_handler,
    create_todos_for_user_handler,
    delete_user_with_todos_handler,
    get_user_with_todos_handler,
    update_user_handler,
    list_users_handler,      
    todoapp_handler,
    user_todos_by_date_handler,    
    update_todos_for_user_handler,
    delete_todo_for_user_handler,    
};

use crate::AppState;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/users/create", post(create_user_handler)) 
        .route("/api/users/{id}/todos", post(create_todos_for_user_handler))
        .route("/api/users/details/{id}", get(get_user_with_todos_handler))
        .route("/api/users/{id}", put(update_user_handler))
        .route("/api/users/todos/{user_id}", put(update_todos_for_user_handler))
        .route("/api/users/delete_with_todos/{id}", delete(delete_user_with_todos_handler))
        .route("/api/users/{user_id}/todos/{todoid}", delete(delete_todo_for_user_handler))
        .route("/api/users", get(list_users_handler)) 
        .route("/api/todoapp", get(todoapp_handler)) 
        .route("/api/users/{user_id}/todos-by-date", get(user_todos_by_date_handler)) 
        .with_state(app_state)
}
