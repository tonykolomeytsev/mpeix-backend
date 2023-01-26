# 🦀 mpeix-backend (v3)

Backend implementation for [**mpeix**](https://github.com/tonykolomeytsev/mpeiapp) written entirely in Rust. The previous two implementations were written in Kotlin, worked well but consumed too much RAM on my cheap server.

This project tries to use Clean Architecture and layering as much as possible.

### Stack

- Rust 🦀 programming language
- Backend framework [**actix-web**](https://github.com/actix/actix-web)
- PostgreSQL for databases
- Docker for packaging applications

### Architecture

As you can see, the whole project is divided into a set of crates. The structure and naming of the crates almost exactly follows the structure and naming of the Gradle modules in the [**mpeix**](https://github.com/tonykolomeytsev/mpeiapp) Android app (which also tries to follow the clean arch as much as possible).

There are four types of crates in the project:
- `app` crates - they are binary crates and are essentially separate Mpeix backend microservices. Each app crate is compiled into a binary, packed into a Docker image, and run on the server.
- `feature` crates - library crates encapsulate access to specific features with complicated business logic. Feature crates could, in theory, be reused between app crates, but there are no examples of such use in this project.
- `domain` crates - library crates encapsulating use-cases, repositories, structures (analogous to dto/pojo from Kotlin/Java).
- `common` crates - library crates that help to reuse application code in other crates. For example, the `common_in_memory_cache` crate contains a wrapper for LRU cache, which is used in many other domain crates.

The dependency rules are also respected: 
- `app` crates shall not depend on other `app` crates (good thing Cargo won't let you do that)
- `feature` crates shall not depend on other `feature` crates;
- `domain` and `common` crates shall not depend on `feature` crates.

All microservices cache all information in the database and on disk as much as possible. For example, the `app_schedule` microservice takes data from the MPEI website and immediately saves it to disk. The old cache is not invalidated at all by design, because the old schedules are deleted from the university's site and invalidating the old cache will cause the data to be lost forever.

A detailed description of each microservice can be found in the README of each `app` crate.
