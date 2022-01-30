use std::borrow::Cow;

use std::collections::HashMap;
use std::io::Write;

use actix_cors::Cors;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::{
    http::header,
    middleware,
    web::{self, Data},
    App, Error, HttpResponse, HttpServer,
};
use clap::Parser;
use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode};
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler};
use mime_guess::from_path;
use rust_embed::RustEmbed;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of times to greet
    #[clap(long)]
    print_schema: bool,
}

#[derive(RustEmbed)]
#[folder = "../frontend/static/"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    if let Some(content) = Asset::get(path) {
        let body: BoxBody = match content.data {
            Cow::Borrowed(bytes) => MessageBody::boxed(bytes),
            Cow::Owned(bytes) => MessageBody::boxed(bytes),
        };
        HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(body)
    } else {
        HttpResponse::NotFound().body("404 Not Found")
    }
}

fn index() -> HttpResponse {
    handle_embedded_file("html/index.html")
}

fn dist(path: web::Path<String>) -> HttpResponse {
    handle_embedded_file(&path)
}
#[derive(Clone, GraphQLObject)]
///a user
pub struct User {
    ///the id
    id: i32,
    ///the name
    name: String,
}

#[derive(Default, Clone)]
pub struct Database {
    ///this could be a database connection
    users: HashMap<i32, User>,
}
impl Database {
    pub fn new() -> Database {
        let mut users = HashMap::new();
        users.insert(
            1,
            User {
                id: 1,
                name: "Aron".to_string(),
            },
        );
        users.insert(
            2,
            User {
                id: 2,
                name: "Bea".to_string(),
            },
        );
        users.insert(
            3,
            User {
                id: 3,
                name: "Carl".to_string(),
            },
        );
        users.insert(
            4,
            User {
                id: 4,
                name: "Dora".to_string(),
            },
        );
        Database { users }
    }
    pub fn get_user(&self, id: &i32) -> Option<&User> {
        self.users.get(id)
    }
}

// To make our Database usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Database {}

// Queries represent the callable funcitons
struct Query;
#[graphql_object(context = Database)]
impl Query {
    fn api_version() -> &'static str {
        "1.0"
    }

    fn user(
        context: &Database,
        #[graphql(description = "id of the user")] id: i32,
    ) -> Option<&User> {
        context.get_user(&id)
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Database>, EmptySubscription<Database>>;

fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<Database>::new(),
        EmptySubscription::<Database>::new(),
    )
}

async fn graphiql_route() -> Result<HttpResponse, Error> {
    graphiql_handler("/graphql", None).await
}

async fn playground_route() -> Result<HttpResponse, Error> {
    playground_handler("/graphql", None).await
}

async fn graphql_route(
    req: actix_web::HttpRequest,
    payload: actix_web::web::Payload,
    schema: web::Data<Schema>,
) -> Result<HttpResponse, Error> {
    let context = Database::new();
    graphql_handler(&schema, &context, req, payload).await
}

fn write_schema() -> std::io::Result<()> {
    use std::fs::File;
    let file = File::create("schema.graphql");
    file?.write_all(schema().as_schema_language().as_bytes())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    if args.print_schema {
        write_schema()?;
        return Result::Ok(());
    }
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
            .service(web::resource("/playground").route(web::get().to(playground_route)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql_route)))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/{_:.*}").route(web::get().to(dist)))
            .service(web::resource("/dist/{_:.*}").route(web::get().to(dist)))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
