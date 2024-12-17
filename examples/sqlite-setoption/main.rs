use sqlx::{
    migrate::Migrator,
    types::chrono::{DateTime, Utc},
    FromRow, SqlitePool,
};
use tiny_orm::{SetOption, Table};

#[derive(Debug, FromRow, Table, Clone)]
struct Todo {
    id: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
    done: bool,
}

#[derive(Debug, FromRow, Table, Clone)]
struct NewTodo {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
    done: bool,
}

#[derive(Debug, Default, FromRow, Table, Clone)]
struct UpdateTodo {
    id: i32,
    updated_at: SetOption<DateTime<Utc>>,
    description: SetOption<String>,
    done: SetOption<bool>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let m = Migrator::new(std::path::Path::new("examples/sqlite/migrations"))
        .await
        .unwrap();
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let _ = m.run(&pool).await.unwrap();

    let new_todo = NewTodo {
        created_at: Utc::now(),
        updated_at: Utc::now(),
        description: "My first item".to_string(),
        done: false,
    };

    let todo = new_todo
        .create(&pool)
        .await
        .expect("Todo item should be created");
    println!("My first todo created {:?}", todo.id);

    let first_todo = Todo::get_by_id(&pool, todo.id).await.unwrap();
    match first_todo {
        Some(ref item) => println!("First todo item is {:?}", item),
        None => println!("Todo item does not exist for the id {}", todo.id),
    }
    let update_todo = UpdateTodo {
        id: todo.id,
        description: "My first item updated".to_string().into(),
        ..Default::default()
    };
    update_todo
        .update(&pool)
        .await
        .expect("Item should be updated");

    let check_updated_item = Todo::get_by_id(&pool, todo.id).await.unwrap();
    match check_updated_item {
        Some(ref item) => println!("Updated item is {:?}", item),
        None => println!("Todo item does not exist for the id {}", todo.id),
    }
}
