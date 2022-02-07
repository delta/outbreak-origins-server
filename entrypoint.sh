#!/bin/bash
while !(pg_isready --dbname="outbreak-origins" --host=db --port=5432 --username=root); do
    sleep 5
done
diesel setup --database-url postgres://root:password@db/"outbreak-origins"
diesel migration run --database-url postgres://root:password@db/"outbreak-origins"
cargo run --release
