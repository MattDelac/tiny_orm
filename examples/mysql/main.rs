use sqlx::{
    migrate::Migrator,
    types::chrono::{DateTime, Utc},
    FromRow, MySqlPool,
};
use tiny_orm::Table;

#[derive(Debug, FromRow, Table, Clone)]
#[tiny_orm(all)]
struct Todo {
    #[tiny_orm(primary_key(auto))]
    id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
    done: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let m = Migrator::new(std::path::Path::new("examples/mysql/migrations"))
        .await
        .unwrap();
    let pool = MySqlPool::connect("mysql://user:password@localhost/examples")
        .await
        .unwrap();
    let _ = m.run(&pool).await.unwrap();

    let new_todo = Todo {
        id: 0, // will be override by the auto increment
        created_at: Utc::now(),
        updated_at: Utc::now(),
        description: "My first item".to_string(),
        done: false,
    };

    let todo_id = new_todo
        .create(&pool)
        .await
        .expect("Todo item should be created");
    println!("My first todo created {:?}", todo_id.clone());

    let first_todo = Todo::get_by_id(&pool, &todo_id).await.unwrap();
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

    let check_updated_item = Todo::get_by_id(&pool, &todo_id).await.unwrap();
    match check_updated_item {
        Some(ref item) => println!("Updated item is {:?}", item),
        None => println!("Todo item does not exist for the id {todo_id}"),
    }

    check_updated_item.unwrap().delete(&pool).await.unwrap();
    let deleted_todo = Todo::get_by_id(&pool, &todo_id).await.unwrap();
    match deleted_todo {
        Some(ref item) => println!("Todo item still exists What??? / {:?}", item),
        None => println!("Todo item has been deleted for the one with the id {todo_id}"),
    }
}
