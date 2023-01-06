use rocket::routes;
use rocket::serde::{json::Json, Serialize};

#[derive(Serialize)]
struct Cat {
    id: u32,
    name: String,
    favorite: bool,
}

#[rocket::get("/cats")]
async fn cats() -> Json<Vec<Cat>> {
    Json(vec![
        Cat {
            id: 123,
            name: "Pika".to_owned(),
            favorite: true,
        },
        Cat {
            id: 124,
            name: "Keksik".to_owned(),
            favorite: false,
        },
    ])
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![cats])
}
