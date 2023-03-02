use postgres::Error as PostgresError;
use postgres::{Client, NoTls};
use serde::Serialize;
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[macro_use]
extern crate serde_derive;

//model: User struct with id, name, email
#[derive(Serialize, Deserialize)]
struct User {
    id: Option<i32>,
    name: String,
    email: String,
}

//DATABASE_URL
const DB_URL: &str = env!("DATABASE_URL");

//constant_response
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";

//main function
fn main() {
    //set database
    if let Err(e) = set_database() {
        println!("Error: {}", e);
        return;
    }

    //start server and print port
    let listener = TcpListener::bind(format!("0.0.0.0:7878")).unwrap();
    println!("Server started at port 7878");

    //handle the client
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream.unwrap());
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

//handle client
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status_line, content) = match &*request {
                r if request_with("POST /users") => handle_post_request(r),
                r if request_with("GET /users/") => handle_get_request(r),
                r if request_with("GET /users") => handle_get_all_request(r),
                r if request_with("PUT /users/") => handle_put_request(r),
                r if request_with("DELETE /users/") => handle_delete_request(r),
                _ => (NOT_FOUND, "Not Found".to_string()),
            };

            stream
                .write_all(format!("{}{}", status_line, content).as_bytes())
                .unwrap();
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
 

//CONTROLLERS

// handle post request function
fn handle_post_request(request: &str) -> (String, String) {
    match (get_user_request_body(&request), Client::connect(DB_URL, NoTls)) {
        (Ok(user), Ok(mut client)) => {
            client.execute("INSERT INTO users (name, email) VALUES ($1, $2)", &[&user.name, &user.email]).unwrap();

            (OK_RESPONSE.to_string(), "User Created".to_string())
        },
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string())
    }
}

//handle get request request
fn handle_get_request(request: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(DB_URL, NoTls)) {
        (Ok(id), Ok(mut client)) => {
            match client.query("SELECT * FROM users WHERE id = $1", &[&id]) {
                Ok(row) => {
                    let user = User {
                            id: row.get(0),
                            name: row.get(1),
                            email: row.get(2),
                        };

                    (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap())
                },
                _ => (NOT_FOUND.to_string(), "User not found".to_string())
            }
        },
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string())
    }
}

//handle get all request
fn handle_get_all_request(_request: &str) -> (String, String) {
    match Client::connect(DB_URL, NoTls) {
        Ok(mut client) => {
            let mut users = Vec::new();

            for row in client.query("SELECT * FROM users", &[]).unwrap() {
                users.push(User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                });
            }

            (OK_RESPONSE.to_string(), serde_json::to_string(&users).unwrap())
        }, 
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string())
    }
}

//handle put request 
fn handle_put_request()


//database connection
fn set_database() -> Result<(), PostgresError> {
    //connect to database
    let mut client = Client::connect(DB_URL, NoTls)?;

    //create table
    client.execute(
        "CREATE TABLE IF NOT EXIST users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        email VARCHAR NOT NULL)",
        &[],
    )?;

    Ok(())
}

//get_id function
fn get_id(request: &str) -> &str {
    request
        .split("/")
        .nth(2)
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or_default()
}

//deserialize user from request body with the id
fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}
