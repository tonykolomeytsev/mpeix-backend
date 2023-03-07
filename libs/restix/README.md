# Restix

Library for code generation of REST Api methods according to the trait description. The API of the Restix library is as close as possible to the API of the Retrofit library from the Android world.

## Introduction

### Simple use case

Restix turns HTTP Api into a trait definition:

```rust
#[api]
pub trait MyApi {
    #[get("/user/{id}")]
    async fn user(&self, id: Path, tag: Query) -> User;
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
let user_id = 12345;
// request to http://localhost:8081/user/0ae2de7d?tag=latest
let user = api.user(user_id, "latest").await?;
```

### Details

The trait `MyApi` will be expanded to:

```rust
#[derive(Clone)]
pub struct MyApi {
    client: reqwest::Client,
    base_url: String,
}

impl MyApi {
    pub fn builder() -> MyApiBuilder {
        MyApiBuilder::default()
    }

    pub async fn user<Path1, Query1>(
        &self, 
        id: Path1, 
        tag: Query1,
    ) -> reqwest::Result<User>
    where
        Path1: Display,
        Query1: AsRef<str>,
    {
        let full_url = format!(
            "{base_url}/user/{id}", 
            base_url = &self.base_url, 
            id = id,
        );
        let queries = vec![
            ("tag", tag.as_ref()),
        ];
        self.client
            .get(full_url)
            .query(&queries)
            .send()
            .await?
            .json::<User>()
            .await
    }
}

// And also MyApiBuilder implementation...
```
