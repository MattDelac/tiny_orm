use sqlx::{
    migrate::Migrator,
    types::chrono::{DateTime, Utc},
    FromRow, PgPool,
};
use tiny_orm::Table;
use uuid::Uuid;

#[allow(unused_variables, dead_code)]
#[derive(Debug, FromRow, Table, Clone)]
#[tiny_orm(table_name = "todo_soft_deleted", soft_deletion)]
struct Todo {
    // Table name would automatically be `todo`. Could be override with `#[tiny_orm(table_name = "xxx")]`
    #[tiny_orm(primary_key)]
    pub id: Uuid, // Or use `#[tiny_orn(primary_key)]` to tell which field to use
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: String,
    pub done: bool,
}
/* Table would generate the following methods
impl Todo {
    pub fn get_by_id(id: &Uuid) -> sqlx::Result<Self> {
        todo!("Get by ID");
    }
    pub fn list_all() -> sqlx::Result<Vec<Self>> {
        todo!("Get by ID");
    }
    pub fn delete(&self) -> sqlx::Result<()> {
        todo!("Delete the item");
    }
}
*/
impl Todo {
    async fn get_soft_deleted_items(pool: &PgPool) -> Result<Vec<Todo>, sqlx::Error> {
        let items = sqlx::query_as::<_, Todo>(
            "SELECT * FROM todo_soft_deleted WHERE deleted_at IS NOT NULL",
        )
        .fetch_all(pool)
        .await?;
        Ok(items)
    }
}

#[derive(Debug, FromRow, Table, Clone)]
#[tiny_orm(table_name = "todo_soft_deleted")]
struct NewTodo {
    description: String,
    done: bool,
}
/* Table would create the following method
impl NewTodo {
    pub fn create(&self) -> sqlx::Result<Todo> {
        todo!("Create a record partially in the database and return it");
    }
}
*/

impl NewTodo {
    pub fn new(description: String) -> Self {
        NewTodo {
            description,
            done: false,
        }
    }
}

#[derive(Debug, FromRow, Table, Clone)]
#[tiny_orm(table_name = "todo_soft_deleted", soft_deletion)]
struct UpdateTodo {
    id: Uuid,
    description: String,
    done: bool,
}
/* Table would create the following method
impl UpdateTodo {
    pub fn update(&self) -> sqlx::Result<Todo> {
        todo!("Update a record partially in the database and return it");
    }
}
*/

impl From<Todo> for UpdateTodo {
    fn from(todo: Todo) -> UpdateTodo {
        UpdateTodo {
            id: todo.id,
            description: todo.description,
            done: todo.done,
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let m = Migrator::new(std::path::Path::new("examples/postgres/migrations"))
        .await
        .unwrap();
    let pool = PgPool::connect("postgres://postgres:password@localhost/examples")
        .await
        .unwrap();
    let _ = m.run(&pool).await.unwrap();

    let new_todo = NewTodo::new("My first item".to_string());
    let todo = new_todo
        .create(&pool)
        .await
        .expect("Todo item should be created");
    println!("My first todo created {:?}", todo);

    let first_todo = Todo::get_by_id(&pool, &todo.id).await.unwrap();
    match first_todo {
        Some(ref item) => println!("First todo item is {:?}", item),
        None => println!("Todo item does not exist for the id {0}", todo.id),
    }

    let mut updated_todo: UpdateTodo = first_todo.unwrap().into();
    updated_todo.done = true;
    let _updated_item = updated_todo
        .update(&pool)
        .await
        .expect("Item should be updated");

    let check_updated_item = Todo::get_by_id(&pool, &todo.id).await.unwrap();
    match check_updated_item {
        Some(ref item) => println!("Updated item is {:?}", item),
        None => println!("Todo item does not exist for the id {0}", todo.id),
    }

    check_updated_item.unwrap().delete(&pool).await.unwrap();
    let deleted_todo = Todo::get_by_id(&pool, &todo.id).await.unwrap();
    match deleted_todo {
        Some(ref item) => println!("Todo item still exists What??? / {:?}", item),
        None => println!(
            "Todo item has been deleted for the one with the id {0}",
            todo.id
        ),
    }

    let soft_deleted_items = Todo::get_soft_deleted_items(&pool).await.unwrap();
    println!("Soft deleted items {:?}", soft_deleted_items);
    assert!(!soft_deleted_items.is_empty());
}
