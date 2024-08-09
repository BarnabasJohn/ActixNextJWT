/*
// Original Todo actix app (actix, postgres, cors,logger) without jwt
// ==================================================================

use actix_cors::Cors;
use actix_web::{ delete, get, http, patch, post, web::{Data, Json, Path}, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use serde::{ Deserialize, Serialize};
use validator::Validate;
use dotenv::dotenv;
use sqlx::{self, postgres::PgPoolOptions, Pool, Postgres, FromRow};



pub struct AppState {
    db: Pool<Postgres>
}

#[derive(Serialize, Deserialize, Validate, FromRow)]
struct Auth {
    #[validate(length(min = 1, message = "auth name required"))]
    name: String,
    email: String,
    password1: String,
    password2: String,
    id: i32,
}


#[derive(Serialize, Deserialize, Validate, FromRow)]
struct Todo {
    #[validate(length(min = 1, message = "todo title required"))]
    title: String,
    content: String,
    creator: i32,
    id: i32,
}

#[derive(Serialize, Deserialize, Validate, FromRow)]
struct CreateUpdateTodo {
    #[validate(length(min = 1, message = "todo title required"))]
    title: String,
    content: String,
}


#[derive(Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Error building a connection pool");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new( move || { let cors = Cors::default()
    //.allowed_origin("http://localhost:3000")
    .allow_any_origin()
    //.allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
    .allow_any_method()
    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
    .allowed_header(http::header::CONTENT_TYPE)
    .max_age(3600);
        
        App::new()
        .wrap(Logger::default())
        .wrap(cors)
        .app_data(Data::new(AppState { db: pool.clone() }))
        .service(get_auths)
        .service(update_auth)
        .service(add_auth)
        .service(get_auth)
        .service(login)
        .service(delete_auth)
        .service(get_todos)
        .service(get_auth_todos)
        .service(get_todo)
        .service(create_todo)
        .service(update_todo)
        .service(delete_todo)

        })
            .bind("127.0.0.1:8080")?
            .run()
            .await
}


#[post("/auth/login")]
async fn login(state: Data<AppState>, login_data: Json<Login>) -> impl Responder {
    let user = sqlx::query_as::<_, Auth>(
        "SELECT * FROM auths WHERE email=$1"
    )
        .bind(&login_data.email)
        .fetch_one(&state.db)
        .await;

    match user {
        Ok(user) => {
            if &login_data.password == &user.password1 {
                HttpResponse::Ok().json(user)
            } else {
                HttpResponse::Unauthorized().finish()
            }
        }
        Err(_) => HttpResponse::Unauthorized().finish(),
    }
}

#[get("/auths")]
async fn get_auths(state: Data<AppState>) -> impl Responder {
    match sqlx::query_as::<_, Auth>(
        "SELECT * FROM auths"
    )
        .fetch_all(&state.db)
        .await
    {
        Ok(auth) => HttpResponse::Ok().json(auth),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get auths"),
    }
}

#[get("/auths/{id}")]
async fn get_auth(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();
    
    match sqlx::query_as::<_, Auth>(
        "SELECT * FROM auths WHERE id = $1"
    )
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(auth) => HttpResponse::Ok().json(auth),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get auth"),
    }
}

#[post("/auths")]
async fn add_auth(state: Data<AppState>, body: Json<Auth>) -> impl Responder {
    let is_valid = body.validate();

    match is_valid {
        Ok(_) => {
            match sqlx::query_as::<_, Auth>(
                "INSERT INTO auths (name, email, password1, password2) VALUES ($1, $2, $3, $4) RETURNING name, email, password1, password2"
            )
                .bind(body.name.to_string())
                .bind(body.email.to_string())
                .bind(body.password1.to_string())
                .bind(body.password2.to_string())
                .fetch_one(&state.db)
                .await
            {
                Ok(auth) => HttpResponse::Ok().json(auth),
                Err(_) => HttpResponse::InternalServerError().json("Failed to create auth"),
            }
        }
        Err(_) => HttpResponse::Ok().body("Auth name is requied!")
    }
    
}

#[patch("/updateauth/{id}")]
async fn update_auth(state: Data<AppState>, body: Json<Auth>, id: Path<i32>) -> impl Responder {
    //let id = id.into_inner().id;
    let id: i32 = id.into_inner();

    let is_valid = body.validate();

    match is_valid {
        Ok(_) => {
            match sqlx::query_as::<_, Auth>(
                "UPDATE auths SET name = $1, email= $2, password1 = $3, password2 = $4 WHERE id = $5 RETURNING name, email"
            )
                .bind(body.name.to_string())
                .bind(body.email.to_string())
                .bind(body.password1.to_string())
                .bind(body.password2.to_string())
                .bind(id)
                .fetch_one(&state.db)
                .await
            {
                Ok(auth) => HttpResponse::Ok().json(auth),
                Err(_) => HttpResponse::InternalServerError().json("Failed to update auth"),
            }
        }
        Err(_) => HttpResponse::Ok().body("Auth name is requied!")
    }
}

#[delete("/delete/{id}")]
async fn delete_auth(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();

    match sqlx::query_as::<_, Auth>(
        "DELETE FROM auths WHERE id = $1 RETURNING name, email"
    )
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(auth) => HttpResponse::Ok().json(auth),
        Err(_) => HttpResponse::InternalServerError().json("Failed to delete auth"),
    }
}

#[get("/todos")]
async fn get_todos(state: Data<AppState>) -> impl Responder {
    match sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos"
    )
        .fetch_all(&state.db)
        .await
    {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get todos"),
    }
}

#[get("/auth/{id}/todos")]
async fn get_auth_todos(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    
    let id: i32 = id.into_inner();

    match sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE creator = $1"
    )
        .bind(id)
        .fetch_all(&state.db)
        .await
    {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get auth's todos"),
    }
}


#[get("/todos/{id}")]
async fn get_todo(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();
    
    match sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE id = $1"
    )
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get todo"),
    }
}

#[post("/auth/{id}/todos")]
async fn create_todo(state: Data<AppState>, body: Json<CreateUpdateTodo>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();

    let is_valid = body.validate();

    match is_valid {
        Ok(_) => {
            match sqlx::query_as::<_, CreateUpdateTodo>(
                "INSERT INTO todos (title, content, creator) VALUES ($1, $2, $3) RETURNING title, content, creator"
            )
                .bind(body.title.to_string())
                .bind(body.content.to_string())
                .bind(id)
                .fetch_one(&state.db)
                .await
            {
                Ok(todo) => HttpResponse::Ok().json(todo),
                Err(_) => HttpResponse::InternalServerError().json("Failed to create todo"),
            }
        }
        Err(_) => HttpResponse::Ok().body("Todo title is requied!")
    }
    
}

#[patch("/updatetodo/{id}")]
async fn update_todo(state: Data<AppState>, body: Json<CreateUpdateTodo>, id: Path<i32>) -> impl Responder {
    //let id = id.into_inner().id;
    let id: i32 = id.into_inner();

    let is_valid = body.validate();

    match is_valid {
        Ok(_) => {
            match sqlx::query_as::<_, CreateUpdateTodo>(
                "UPDATE todos SET title = $1, content= $2 WHERE id = $3 RETURNING title, content"
            )
                .bind(body.title.to_string())
                .bind(body.content.to_string())
                .bind(id)
                .fetch_one(&state.db)
                .await
            {
                Ok(todo) => HttpResponse::Ok().json(todo),
                Err(_) => HttpResponse::InternalServerError().json("Failed to update todo"),
            }
        }
        Err(_) => HttpResponse::Ok().body("Todo title is requied!")
    }
}

#[delete("/delete_todo/{id}")]
async fn delete_todo(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();

    match sqlx::query_as::<_, Todo>(
        "DELETE FROM todos WHERE id = $1"
    )
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(err) => {
            eprint!("Error deleting todo: {:?}", err);
            HttpResponse::InternalServerError().json("Failed to delete todo")
        }
    }
}
*/


//Todo-Actix with implementation of jwt
//This is an improved version holding tokens within the request headers of protected routes

use actix_cors::Cors;
use actix_web::{ delete, get, patch, post,
                web::{Data, Json, Path}, App,
                HttpResponse, HttpServer, Responder, 
                middleware::Logger, dev::Payload, error::ErrorUnauthorized,
                http::{self, header::HeaderValue},
                Error as ActixWebError, FromRequest
            };
use serde::{ Deserialize, Serialize};
use validator::Validate;
use dotenv::dotenv;
use sqlx::{self, postgres::PgPoolOptions, Pool, Postgres, FromRow};
use jsonwebtoken::{ encode, decode,Algorithm, Header,
                    EncodingKey, DecodingKey, Validation,
                    TokenData, errors::Error as JwtError};
use chrono::{Utc, Duration};
use std::future::{ Ready, ready };



pub struct AppState {
    db: Pool<Postgres>
}

#[derive(Serialize, Deserialize, Validate, FromRow)]
struct Auth {
    #[validate(length(min = 1, message = "auth name required"))]
    name: String,
    email: String,
    password1: String,
    password2: String,
    id: i32,
}


#[derive(Serialize, Deserialize, Validate, FromRow)]
struct Todo {
    #[validate(length(min = 1, message = "todo title required"))]
    title: String,
    content: String,
    creator: i32,
    id: i32,
}

#[derive(Serialize, Deserialize, Validate, FromRow)]
struct CreateUpdateTodo {
    #[validate(length(min = 1, message = "todo title required"))]
    title: String,
    content: String,
}


#[derive(Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: i32,
    pub exp: usize,
}


#[derive(Serialize, Deserialize)]
struct EncodedResponse {
    message: String,
    token: String,
    user: Auth,
}

#[derive(Serialize, Deserialize)]
struct Response {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct DecodeToken {
    token: String
}

#[derive(Serialize, Deserialize)]
struct DecodedResponse {
    message: String,
    id: i32,
}

#[derive(Serialize, Deserialize)]
struct AuthenticationToken {
    id: i32,
}

impl FromRequest for AuthenticationToken {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut Payload) -> Self::Future {
        //get auth token from authorization header
        let auth_header: Option<&HeaderValue> = req.headers().get(http::header::AUTHORIZATION);
        let auth_token: String = auth_header.unwrap().to_str().unwrap_or("").to_string();
        if auth_token.is_empty() { return ready(Err(ErrorUnauthorized("Invalid auth token!"))); }

        //let secret: String = req.app_data::Data<String>().unwrap().to_string();
        let secret: &str = &req.app_data::<Data<String>>().unwrap();

        //let decode token with the secret
        let decode: Result<TokenData<Claims>, JwtError> = decode::<Claims>(
            &auth_token.trim(),
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        );
        //return authentication token
        match decode {
            Ok(token)=> ready(Ok(AuthenticationToken { id: token.claims.id})),
            Err(e) => ready(Err(ErrorUnauthorized(e.to_string()))),
        }
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Error building a connection pool");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new( move || { let cors = Cors::default()
    //.allowed_origin("http://localhost:3000")
    .allow_any_origin()
    //.allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
    .allow_any_method()
    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
    .allowed_header(http::header::CONTENT_TYPE)
    .max_age(3600);
        
        App::new()
        .wrap(Logger::default())
        .wrap(cors)
        .app_data(Data::new(AppState { db: pool.clone() }))
        .app_data(Data::new(String::from("secret")))
        .service(get_auths)
        .service(update_auth)
        .service(add_auth)
        .service(get_auth)
        .service(login)
        .service(delete_auth)
        .service(get_todos)
        .service(get_auth_todos)
        .service(get_todo)
        .service(create_todo)
        .service(update_todo)
        .service(delete_todo)
        .service(decode_token)

        })
            .bind("127.0.0.1:8080")?
            .run()
            .await
}


#[post("/auth/login")]
async fn login(state: Data<AppState>, login_data: Json<Login>, secret: Data<String>) -> impl Responder {
    let user = sqlx::query_as::<_, Auth>(
        "SELECT * FROM auths WHERE email=$1"
    )
        .bind(&login_data.email)
        .fetch_one(&state.db)
        .await;

    match user {
        Ok(user) => {
            if &login_data.password == &user.password1 {
                let id: i32 = user.id;
                let exp: usize = (Utc::now() + Duration::minutes(30)).timestamp() as usize;
                let claims: Claims = Claims { id, exp };

                let token: String = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(secret.as_str().as_ref())
                ).unwrap();

                HttpResponse::Ok().json(EncodedResponse {
                    message: "successfully created token".to_owned(),
                    token,
                    user
                })
            } else {
                HttpResponse::Unauthorized().finish()
            }
        }
        Err(_) => HttpResponse::Unauthorized().finish(),
    }
}

#[get("/auths")]
async fn get_auths(state: Data<AppState>) -> impl Responder {
    match sqlx::query_as::<_, Auth>(
        "SELECT * FROM auths"
    )
        .fetch_all(&state.db)
        .await
    {
        Ok(auth) => HttpResponse::Ok().json(auth),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get auths"),
    }
}

#[get("/auths/{id}")]
async fn get_auth(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();
    
    match sqlx::query_as::<_, Auth>(
        "SELECT * FROM auths WHERE id = $1"
    )
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(auth) => HttpResponse::Ok().json(auth),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get auth"),
    }
}

#[post("/auths")]
async fn add_auth(state: Data<AppState>, body: Json<Auth>) -> impl Responder {
    let is_valid = body.validate();

    match is_valid {
        Ok(_) => {
            match sqlx::query_as::<_, Auth>(
                "INSERT INTO auths (name, email, password1, password2) VALUES ($1, $2, $3, $4) RETURNING name, email, password1, password2"
            )
                .bind(body.name.to_string())
                .bind(body.email.to_string())
                .bind(body.password1.to_string())
                .bind(body.password2.to_string())
                .fetch_one(&state.db)
                .await
            {
                Ok(auth) => HttpResponse::Ok().json(auth),
                Err(_) => HttpResponse::InternalServerError().json("Failed to create auth"),
            }
        }
        Err(_) => HttpResponse::Ok().body("Auth name is requied!")
    }
    
}

#[patch("/updateauth/{id}")]
async fn update_auth(state: Data<AppState>, body: Json<Auth>, id: Path<i32>) -> impl Responder {
    //let id = id.into_inner().id;
    let id: i32 = id.into_inner();

    let is_valid = body.validate();

    match is_valid {
        Ok(_) => {
            match sqlx::query_as::<_, Auth>(
                "UPDATE auths SET name = $1, email= $2, password1 = $3, password2 = $4 WHERE id = $5 RETURNING name, email"
            )
                .bind(body.name.to_string())
                .bind(body.email.to_string())
                .bind(body.password1.to_string())
                .bind(body.password2.to_string())
                .bind(id)
                .fetch_one(&state.db)
                .await
            {
                Ok(auth) => HttpResponse::Ok().json(auth),
                Err(_) => HttpResponse::InternalServerError().json("Failed to update auth"),
            }
        }
        Err(_) => HttpResponse::Ok().body("Auth name is requied!")
    }
}

#[delete("/delete/{id}")]
async fn delete_auth(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();

    match sqlx::query_as::<_, Auth>(
        "DELETE FROM auths WHERE id = $1 RETURNING name, email"
    )
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(auth) => HttpResponse::Ok().json(auth),
        Err(_) => HttpResponse::InternalServerError().json("Failed to delete auth"),
    }
}

#[post("/decode")]
async fn decode_token(body: Json<DecodeToken>, secret: Data<String>) -> HttpResponse {
    let decoded: Result<TokenData<Claims>, JwtError> = decode::<Claims>(
        &body.token,
        &DecodingKey::from_secret(secret.as_str().as_ref()),
        &Validation::new(Algorithm::HS256)
    );

    match decoded {
        Ok(token) => HttpResponse::Ok().json(DecodedResponse {
            message: ("Authorized!").to_owned(),
            id: token.claims.id,
        }),
        Err(e) => HttpResponse::BadRequest().json( Response { message: e.to_string() }),
    }
}

#[get("/todos")]
async fn get_todos(state: Data<AppState>, auth_token: AuthenticationToken) -> impl Responder {
    match sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos"
    )
        .fetch_all(&state.db)
        .await
    {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get todos"),
    }
}

#[get("/auth/{id}/todos")]
async fn get_auth_todos(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    
    let id: i32 = id.into_inner();

    match sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE creator = $1"
    )
        .bind(id)
        .fetch_all(&state.db)
        .await
    {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get auth's todos"),
    }
}


#[get("/todos/{id}")]
async fn get_todo(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();
    
    match sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE id = $1"
    )
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get todo"),
    }
}

#[post("/auth/{id}/todos")]
async fn create_todo(state: Data<AppState>, body: Json<CreateUpdateTodo>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();

    let is_valid = body.validate();

    match is_valid {
        Ok(_) => {
            match sqlx::query_as::<_, CreateUpdateTodo>(
                "INSERT INTO todos (title, content, creator) VALUES ($1, $2, $3) RETURNING title, content, creator"
            )
                .bind(body.title.to_string())
                .bind(body.content.to_string())
                .bind(id)
                .fetch_one(&state.db)
                .await
            {
                Ok(todo) => HttpResponse::Ok().json(todo),
                Err(_) => HttpResponse::InternalServerError().json("Failed to create todo"),
            }
        }
        Err(_) => HttpResponse::Ok().body("Todo title is requied!")
    }
    
}

#[patch("/updatetodo/{id}")]
async fn update_todo(state: Data<AppState>, body: Json<CreateUpdateTodo>, id: Path<i32>) -> impl Responder {
    //let id = id.into_inner().id;
    let id: i32 = id.into_inner();

    let is_valid = body.validate();

    match is_valid {
        Ok(_) => {
            match sqlx::query_as::<_, CreateUpdateTodo>(
                "UPDATE todos SET title = $1, content= $2 WHERE id = $3 RETURNING title, content"
            )
                .bind(body.title.to_string())
                .bind(body.content.to_string())
                .bind(id)
                .fetch_one(&state.db)
                .await
            {
                Ok(todo) => HttpResponse::Ok().json(todo),
                Err(_) => HttpResponse::InternalServerError().json("Failed to update todo"),
            }
        }
        Err(_) => HttpResponse::Ok().body("Todo title is requied!")
    }
}

#[delete("/delete_todo/{id}")]
async fn delete_todo(state: Data<AppState>, id: Path<i32>) -> impl Responder {
    let id: i32 = id.into_inner();

    match sqlx::query_as::<_, Todo>(
        "DELETE FROM todos WHERE id = $1"
    )
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(err) => {
            eprint!("Error deleting todo: {:?}", err);
            HttpResponse::InternalServerError().json("Failed to delete todo")
        }
    }
}

// async fn create_jwt(user_id: i32, secret: Data<String>) -> String {
//     let expiration = (Utc::now()+ Duration::minutes(30)).timestamp() as usize;

//     let claims = Claims {
//         sub: user_id,
//         exp: expiration as usize,
//     };

//     encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_str().as_ref()))
//         .expect("Failed to create token")

// }

// async fn decode_jwt(token: &str, secret: Data<String>) -> Result<Claims, jsonwebtoken::errors::Error> {
//     decode::<Claims>(token, &DecodingKey::from_secret(secret.as_str().as_ref()), &Validation::default())
//         .map(|data| data.claims)
// }





