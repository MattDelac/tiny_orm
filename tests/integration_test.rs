use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
    SqlitePool,
};
use tiny_orm::Table;

#[derive(Debug, PartialEq, Table, FromRow)]
#[tiny_orm(exclude = "create", add = "update")]
struct Todo {
    id: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
    done: bool,
}
impl Todo {
    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn change_description(&mut self, description: String) {
        self.description = description
    }
}

#[derive(Debug, PartialEq, Table)]
struct NewTodo {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
}
impl NewTodo {
    fn new(description: String) -> Self {
        Self {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            description,
        }
    }
}

#[tokio::test]
async fn test_table_name() {
    assert_eq!(Todo::table_name(), "todo");
}

#[sqlx::test(migrations = "examples/sqlite/migrations")]
async fn test_insert_and_get_by_id(pool: SqlitePool) {
    let description = "My new item".to_string();
    let new_item = NewTodo::new(description.clone());

    let inserted_item = new_item.create(&pool).await.unwrap();
    assert!(inserted_item.id > -0);
    assert!(inserted_item.created_at < Utc::now());
    assert!(inserted_item.updated_at < Utc::now());
    assert_eq!(inserted_item.description, description);

    let checked_item = Todo::get_by_id(&pool, inserted_item.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(inserted_item, checked_item);
}

#[sqlx::test(migrations = "examples/sqlite/migrations")]
async fn test_update(pool: SqlitePool) {
    let description = "My new item".to_string();
    let new_item = NewTodo::new(description.clone());

    let inserted_item = new_item.create(&pool).await.unwrap();
    assert_eq!(inserted_item.description, description);

    let mut updated_item = inserted_item;
    updated_item.description = "New description".to_string();
    updated_item.done = true;
    updated_item.update(&pool).await.unwrap();

    let checked_item = Todo::get_by_id(&pool, updated_item.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_item, checked_item);
    assert_eq!(updated_item.description, "New description".to_string());
    assert!(updated_item.done);
}

#[sqlx::test(migrations = "examples/sqlite/migrations")]
async fn test_list_all(pool: SqlitePool) {
    let _ = NewTodo::new("Item 1".to_string())
        .create(&pool)
        .await
        .unwrap();
    let _ = NewTodo::new("Item 2".to_string())
        .create(&pool)
        .await
        .unwrap();
    let _ = NewTodo::new("Item 3".to_string())
        .create(&pool)
        .await
        .unwrap();

    let all_items: Vec<Todo> = Todo::list_all(&pool).await.unwrap();
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
    let item = NewTodo::new("Item 1".to_string())
        .create(&pool)
        .await
        .unwrap();

    let retrieved_item = Todo::get_by_id(&pool, item.id).await.unwrap();
    assert!(retrieved_item.is_some());

    item.delete(&pool).await.unwrap();
    let retrieved_item = Todo::get_by_id(&pool, item.id).await.unwrap();
    assert!(retrieved_item.is_none());
}

#[sqlx::test(migrations = "examples/sqlite/migrations")]
async fn test_insert_get_with_a_transaction(pool: SqlitePool) {
    let mut tx = pool.begin().await.unwrap();
    let description = "My new item".to_string();
    let new_item = NewTodo::new(description.clone());

    let mut inserted_item = new_item.create(&mut *tx).await.unwrap();
    assert!(inserted_item.id > -0);
    assert!(inserted_item.created_at < Utc::now());
    assert!(inserted_item.updated_at < Utc::now());
    assert_eq!(inserted_item.description, description);

    inserted_item.change_description("New description".to_string());
    inserted_item.update(&mut *tx).await.unwrap();

    tx.commit().await.unwrap();

    let checked_item = Todo::get_by_id(&pool, inserted_item.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(checked_item.description(), "New description");
}
