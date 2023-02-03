
mkdir -p $1/src
touch $1/src/lib.rs
echo '[package]
name = "'$1'"
version = "0.1.0"
edition = "2021"
authors = ["Anton Kolomeytsev <tonykolomeytsev@gmail.com>"]' > $1/Cargo.toml
