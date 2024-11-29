use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow, Row, SqlitePool,
};
use tiny_orm::TinyORM;

#[derive(Debug, FromRow, TinyORM, Clone)]
struct Todos {
    id: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
    done: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let pool = SqlitePool::connect("sqlite:simple.db")
        .await
        .expect("SqlitePool should be created");

    let new_todo = Todos {
        id: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        description: "My first item".to_string(),
        done: false,
    };

    let todo_id = new_todo
        .create(&pool)
        .await
        .expect("Todo item should be created");
    println!("My first todo created {:?}", todo_id);

    let first_todo = Todos::get_by_id(&pool, todo_id).await.unwrap();
    match first_todo {
        Some(ref item) => println!("First todo item is {:?}", item),
        None => println!("Todo item does not exist for the id {todo_id}"),
    }

    let mut updated_todo = first_todo.unwrap();
    updated_todo.done = true;
    updated_todo
        .update(&pool)
        .await
        .expect("Item should be updated");

    let check_updated_item = Todos::get_by_id(&pool, todo_id).await.unwrap();
    match check_updated_item {
        Some(ref item) => println!("Updated item is {:?}", item),
        None => println!("Todo item does not exist for the id {todo_id}"),
    }

    check_updated_item.unwrap().delete(&pool).await.unwrap();
    let deleted_todo = Todos::get_by_id(&pool, todo_id).await.unwrap();
    match deleted_todo {
        Some(ref item) => println!("Todo item still exists What??? / {:?}", item),
        None => println!("Todo item has been deleted for the one with the id {todo_id}"),
    }
}
