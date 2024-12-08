use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
    SqlitePool,
};
use tiny_orm::TinyORM;

#[derive(Debug, PartialEq, TinyORM, FromRow)]
#[tiny_orm(exclude = "create")]
struct Todos {
    id: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
    done: bool,
}

#[derive(Debug, PartialEq, TinyORM)]
#[tiny_orm(table_name = "todos", only = "create", return_object = "Todos")]
struct NewTodos {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
}
impl NewTodos {
    fn new(description: String) -> Self {
        Self {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            description,
        }
    }
}

#[sqlx::test(migrations = "examples/sqlite/migrations")]
async fn test_insert_and_get_by_id(pool: SqlitePool) {
    let description = "My new item".to_string();
    let new_item = NewTodos::new(description.clone());

    let inserted_item = new_item.create(&pool).await.unwrap();
    assert!(inserted_item.id > -0);
    assert!(inserted_item.created_at < Utc::now());
    assert!(inserted_item.updated_at < Utc::now());
    assert_eq!(inserted_item.description, description);

    let checked_item = Todos::get_by_id(&pool, inserted_item.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(inserted_item, checked_item);
}

#[sqlx::test(migrations = "examples/sqlite/migrations")]
async fn test_update(pool: SqlitePool) {
    let description = "My new item".to_string();
    let new_item = NewTodos::new(description.clone());

    let inserted_item = new_item.create(&pool).await.unwrap();
    assert_eq!(inserted_item.description, description);

    let mut updated_item = inserted_item;
    updated_item.description = "New description".to_string();
    updated_item.done = true;
    updated_item.update(&pool).await.unwrap();

    let checked_item = Todos::get_by_id(&pool, updated_item.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_item, checked_item);
    assert_eq!(updated_item.description, "New description".to_string());
    assert!(updated_item.done);
}

#[sqlx::test(migrations = "examples/sqlite/migrations")]
async fn test_list_all(pool: SqlitePool) {
    let _ = NewTodos::new("Item 1".to_string())
        .create(&pool)
        .await
        .unwrap();
    let _ = NewTodos::new("Item 2".to_string())
        .create(&pool)
        .await
        .unwrap();
    let _ = NewTodos::new("Item 3".to_string())
        .create(&pool)
        .await
        .unwrap();

    let all_items: Vec<Todos> = Todos::list_all(&pool).await.unwrap();
    assert_eq!(all_items.len(), 3);

    let mut all_descriptions: Vec<String> = all_items.into_iter().map(|x| x.description).collect();
    all_descriptions.sort();
    assert_eq!(
        all_descriptions,
        vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
            "Item 3".to_string()
        ]
    );
}

#[sqlx::test(migrations = "examples/sqlite/migrations")]
async fn test_delete(pool: SqlitePool) {
    let item = NewTodos::new("Item 1".to_string())
        .create(&pool)
        .await
        .unwrap();

    let retrieved_item = Todos::get_by_id(&pool, item.id).await.unwrap();
    assert!(retrieved_item.is_some());

    item.delete(&pool).await.unwrap();
    let retrieved_item = Todos::get_by_id(&pool, item.id).await.unwrap();
    assert!(retrieved_item.is_none());
}
