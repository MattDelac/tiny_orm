use sqlx::{
    migrate::Migrator,
    types::chrono::{DateTime, Utc},
    FromRow, PgPool,
};
use tiny_orm::TinyORM;
use uuid::Uuid;

#[allow(unused_variables, dead_code)]
#[derive(Debug, FromRow, TinyORM, Clone)]
#[tiny_orm(exclude = "create,update")]
struct Todos {
    // Table name would automatically be `todos`. Could be override with `#[tiny_orm(table_name = "xxx")]`
    #[tiny_orm(primary_key)]
    pub id: Uuid, // Or use `#[tiny_orn(primary_key)]` to tell which field to use
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub description: String,
    pub done: bool,
}
/* TinyORM would generate the following methods
impl Todos {
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

#[derive(Debug, FromRow, TinyORM, Clone)]
#[tiny_orm(table_name = "todos", return_object = "Todos", only = "create")]
struct NewTodos {
    description: String,
    done: bool,
}
/* TinyORM would create the following method
impl NewTodos {
    pub fn create(&self) -> sqlx::Result<Todos> {
        todo!("Create a record partially in the database and return it");
    }
}
*/

impl NewTodos {
    pub fn new(description: String) -> Self {
        NewTodos {
            description,
            done: false,
        }
    }
}

#[derive(Debug, FromRow, TinyORM, Clone)]
#[tiny_orm(table_name = "todos", return_object = "Todos", only = "update")]
struct UpdateTodos {
    id: Uuid,
    description: String,
    done: bool,
}
/* TinyORM would create the following method
impl UpdateTodos {
    pub fn update(&self) -> sqlx::Result<Todos> {
        todo!("Update a record partially in the database and return it");
    }
}
*/

impl From<Todos> for UpdateTodos {
    fn from(todo: Todos) -> UpdateTodos {
        UpdateTodos {
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

    let new_todo = NewTodos::new("My first item".to_string());
    let todo = new_todo
        .create(&pool)
        .await
        .expect("Todo item should be created");
    println!("My first todo created {:?}", todo);

    let first_todo = Todos::get_by_id(&pool, todo.id).await.unwrap();
    match first_todo {
        Some(ref item) => println!("First todo item is {:?}", item),
        None => println!("Todo item does not exist for the id {0}", todo.id),
    }

    let mut updated_todo: UpdateTodos = first_todo.unwrap().into();
    updated_todo.done = true;
    let _updated_item = updated_todo
        .update(&pool)
        .await
        .expect("Item should be updated");

    let check_updated_item = Todos::get_by_id(&pool, todo.id).await.unwrap();
    match check_updated_item {
        Some(ref item) => println!("Updated item is {:?}", item),
        None => println!("Todo item does not exist for the id {0}", todo.id),
    }

    check_updated_item.unwrap().delete(&pool).await.unwrap();
    let deleted_todo = Todos::get_by_id(&pool, todo.id).await.unwrap();
    match deleted_todo {
        Some(ref item) => println!("Todo item still exists What??? / {:?}", item),
        None => println!(
            "Todo item has been deleted for the one with the id {0}",
            todo.id
        ),
    }
}
