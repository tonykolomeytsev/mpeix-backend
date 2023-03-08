# Restix

Library for code generation of REST Api methods according to the trait description. The API of the Restix library is as close as possible to the API of the Retrofit library from the Android world.

## Introduction

### Simple use case

Restix turns HTTP Api into a trait definition:

```rust
#[api]
pub trait MyApi {
    #[get("/user/{id}")]
    async fn user(&self, #[path] id: i64, #[query] tag: &str) -> User;
}
```

Attribute macro `#[api]` will generate `MyApi` struct with method implementation and `MyApiBuilder` struct:

```rust
let api = MyApi::builder()
    .base_url("http://localhost:8080")
    .client(reqwest::Client::default())
    .build()
    .unwrap();
```

Then you can use `api` to make requests:

```rust
// request to http://localhost:8081/user/12345?tag=latest
let user = api.user(12345, "latest").await?;
```

## Api declaration

Attributes on the trait methods and its arguments indicate how request implementation will be generated.

### Request method

Every method should be marked with attribute macros: `#[get("...")]`, `#[post("...")]`, and others.
The relative URL of the resource is specified in the attributes:
```rust
#[get("/users/list")]
```

### URL manipulation

Request URL can be updated dynamically using format blocks in the URL and arguments in the method:
```rust
#[get("/group/{id}/users")]
async fn get_group_users(&self, #[path] id: i64) -> Vec<User>;
```

Query parameters can also be added:
```rust
#[get("/group/{id}/users")]
async fn get_group_users(&self, #[path] id: i64, #[query] sort: &str) -> Vec<User>;
```

### Request body

An argument can be specified for use as an HTTP request body with the `#[body]` attribute.
Argument type must implement `serde::Serialize`:
```rust
#[post("/group/create")]
async fn create_group(&self, #[body] group: Group) -> Group;
```

## Features

By default Restix uses `"reqwest"` and `"json"` features. This means that the generated Api implementations use `reqwest` for requests and `serde` for deserializing responses.
