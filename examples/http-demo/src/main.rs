use anyhow::Result;

use http::*;
use kv::*;
use slight_http_handler_macro::register_handler;

wit_bindgen_rust::import!("../../wit/http.wit");
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(http::Error);
wit_error_rs::impl_error!(kv::Error);

fn main() -> Result<()> {
    let router = Router::new()?;
    let router_with_route = router
        .get("/hello", "handle_hello")?
        .get("/foo", "handle_foo")?
        .put("/bar", "handle_bar")?
        .post("/upload", "upload")?
        .delete("/delete-file", "delete_file_handler")?;

    println!("guest starting server");
    let _ = Server::serve("0.0.0.0:3000", &router_with_route)?;
    // server.stop().unwrap();
    println!("guest moving on");

    Ok(())
}

#[register_handler]
fn handle_hello(req: Request) -> Result<Response, Error> {
    Ok(Response {
        headers: Some(req.headers),
        body: Some("hello".as_bytes().to_vec()),
        status: 200,
    })
}

#[register_handler]
fn handle_foo(request: Request) -> Result<Response, Error> {
    let kv = crate::Kv::open("my-container").unwrap();
    let value = kv.get("key").unwrap();
    Ok(Response {
        headers: Some(request.headers),
        body: Some(value),
        status: 500,
    })
}

#[register_handler]
fn handle_bar(request: Request) -> Result<Response, Error> {
    assert_eq!(request.method, Method::Put);
    println!("request body: {:?}", request.body);
    if let Some(body) = request.body {
        let kv = crate::Kv::open("my-container").unwrap();
        println!("here1");
        kv.set("key", &body).unwrap();
        println!("here2");
    }
    Ok(Response {
        headers: Some(request.headers),
        body: None,
        status: 200,
    })
}

#[register_handler]
fn delete_file_handler(request: Request) -> Result<Response, Error> {
    assert_eq!(request.method, Method::Delete);
    Ok(Response {
        headers: Some(request.headers),
        body: request.body,
        status: 200,
    })
}


#[register_handler]
fn upload(request: Request) -> Result<Response, Error> {
    assert_eq!(request.method, Method::Post);
    Ok(Response {
        headers: Some(request.headers),
        body: request.body,
        status: 200,
    })
}
