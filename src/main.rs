#![feature(proc_macro_hygiene, decl_macro)]
#![allow(dead_code)]

#[macro_use]
extern crate rocket;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use std::path::{Path, PathBuf};

use rocket::http::Method;
use rocket::response::NamedFile;
use rocket::State;

use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Error};

use serde::{Deserialize, Serialize};

use uuid::Uuid;

use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: String,
    content: String,
    completed: bool,
}

type Todos = Vec<Todo>;

#[derive(Debug, Serialize, Deserialize)]
struct TodoAppState {
    todos: Todos,
}

impl TodoAppState {
    fn new() -> TodoAppState {
        TodoAppState { todos: vec![] }
    }

    fn add_todo(&mut self, content: &str) -> String {
        let uuid = Uuid::new_v4();
        let todo = Todo {
            id: uuid.to_hyphenated().to_string(),
            content: content.to_owned(),
            completed: false,
        };
        let todo_id = todo.id.clone();
        self.todos.push(todo);
        todo_id
    }

    fn remove_todo(&mut self, todo_id: &str) -> bool {
        let len = self.todos.len();
        self.todos.retain(|todo| todo.id != todo_id);
        len != self.todos.len()
    }

    fn clear_completed(&mut self) {
        self.todos.retain(|todo| !todo.completed)
    }

    fn handle_todo<F>(&mut self, todo_id: &str, mut handler: F) -> bool
    where
        F: FnMut(&mut Todo) -> (),
    {
        for todo in &mut self.todos {
            if todo.id == todo_id {
                handler(todo);
                return true;
            }
        }
        return false;
    }

    fn update_todo(&mut self, todo_id: &str, todo_content: &str) -> bool {
        self.handle_todo(todo_id, move |todo| {
            *todo = Todo {
                id: todo_id.to_owned(),
                content: todo_content.to_owned(),
                completed: todo.completed,
            };
        })
    }

    fn toggle_todo(&mut self, todo_id: &str) -> bool {
        self.handle_todo(todo_id, |todo| {
            todo.completed = !todo.completed;
        })
    }

    fn toggle_all(&mut self) {
        let mut all_completed = true;

        for todo in &self.todos {
            if !todo.completed {
                all_completed = false;
                break;
            }
        }

        for todo in &mut self.todos {
            (*todo).completed = !all_completed;
        }
    }
}

static STATIC_DIRECTORY: &'static str = "static/";

fn get_static_file(filename: PathBuf) -> PathBuf {
    Path::new(STATIC_DIRECTORY).join(filename)
}

#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open("index.html").ok()
}

#[derive(Serialize)]
struct TodosResponse {
    success: bool,
    message: String,
    data: Todos,
}

#[get("/todos?<filter>")]
fn todos(filter: Option<String>, state: State<Mutex<TodoAppState>>) -> Json<TodosResponse> {
    let todo_app_state = state.lock().unwrap();
    let mut todos = todo_app_state.todos.to_vec();

    todos.retain(|todo| {
        if let Some(filter) = &filter {
            match &filter[..] {
                "all" => true,
                "active" => !todo.completed,
                "completed" => todo.completed,
                _ => true,
            }
        } else {
            true
        }
    });

    Json(TodosResponse {
        success: true,
        message: "".to_owned(),
        data: todos,
    })
}

#[derive(Deserialize)]
struct AddTodoPayload {
    content: String,
}

#[derive(Serialize)]
struct AddTodoResponse {
    success: bool,
    message: String,
}

#[post("/add_todo", format = "json", data = "<payload>")]
fn add_todo(
    payload: Json<AddTodoPayload>,
    state: State<Mutex<TodoAppState>>,
) -> Json<AddTodoResponse> {
    let mut todo_app_state = state.lock().unwrap();
    let response = AddTodoResponse {
        success: true,
        message: "".to_owned(),
    };
    todo_app_state.add_todo(&payload.content);
    Json(response)
}

#[derive(Deserialize)]
struct RemoveTodoPayload {
    todo_id: String,
}

#[derive(Serialize)]
struct RemoveTodoResponse {
    success: bool,
    message: String,
}

#[post("/remove_todo", format = "json", data = "<payload>")]
fn remove_todo(
    payload: Json<RemoveTodoPayload>,
    state: State<Mutex<TodoAppState>>,
) -> Json<RemoveTodoResponse> {
    let mut todo_app_state = state.lock().unwrap();

    match todo_app_state.remove_todo(&payload.todo_id) {
        true => Json(RemoveTodoResponse {
            success: true,
            message: "".to_owned(),
        }),
        false => Json(RemoveTodoResponse {
            success: false,
            message: format!("{id} is not found", id = payload.todo_id),
        }),
    }
}

#[derive(Deserialize)]
struct UpdateTodoPayload {
    todo_id: String,
    content: String,
}

#[derive(Serialize)]
struct UpdateTodoResponse {
    success: bool,
    message: String,
}

#[post("/update_todo", format = "json", data = "<payload>")]
fn update_todo(
    payload: Json<UpdateTodoPayload>,
    state: State<Mutex<TodoAppState>>,
) -> Json<UpdateTodoResponse> {
    let mut todo_app_state = state.lock().unwrap();
    match todo_app_state.update_todo(&payload.todo_id, &payload.content) {
        true => Json(UpdateTodoResponse {
            success: true,
            message: "".to_owned(),
        }),
        false => Json(UpdateTodoResponse {
            success: false,
            message: format!("{id} is not found", id = &payload.todo_id),
        }),
    }
}

#[derive(Deserialize)]
struct ToggleTodoPayload {
    todo_id: String,
}

#[derive(Serialize)]
struct ToggleTodoResponse {
    success: bool,
    message: String,
}

#[post("/toggle_todo", format = "json", data = "<payload>")]
fn toggle_todo(
    payload: Json<ToggleTodoPayload>,
    state: State<Mutex<TodoAppState>>,
) -> Json<ToggleTodoResponse> {
    let mut todo_app_state = state.lock().unwrap();
    match todo_app_state.toggle_todo(&payload.todo_id) {
        true => Json(ToggleTodoResponse {
            success: true,
            message: "".to_owned(),
        }),
        false => Json(ToggleTodoResponse {
            success: false,
            message: format!("{id} is not found!", id = payload.todo_id),
        }),
    }
}

#[derive(Serialize)]
struct ClearCompletedResponse {
    success: bool,
    message: String,
}

#[post("/clear_completed")]
fn clear_completed(state: State<Mutex<TodoAppState>>) -> Json<ClearCompletedResponse> {
    let mut todo_app_state = state.lock().unwrap();
    todo_app_state.clear_completed();
    Json(ClearCompletedResponse {
        success: true,
        message: "".to_owned(),
    })
}

#[derive(Serialize)]
struct ToggleAllResponse {
    success: bool,
    message: String,
}

#[post("/toggle_all")]
fn toggle_all(state: State<Mutex<TodoAppState>>) -> Json<ToggleAllResponse> {
    let mut todo_app_state = state.lock().unwrap();
    todo_app_state.toggle_all();
    Json(ToggleAllResponse {
        success: true,
        message: "".to_owned(),
    })
}

fn main() -> Result<(), Error> {
    let allowed_origins = AllowedOrigins::some_regex(&["^https?://localhost"]);

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;

    let handlers = routes![
        todos,
        add_todo,
        remove_todo,
        update_todo,
        toggle_todo,
        clear_completed,
        toggle_all
    ];
    rocket::ignite()
        .mount("/", handlers)
        .mount("/static", StaticFiles::from(STATIC_DIRECTORY))
        .manage(Mutex::new(TodoAppState::new()))
        .attach(cors)
        .launch();

    Ok(())
}
