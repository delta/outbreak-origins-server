FROM rust:latest
RUN apt-get update && apt-get install -y postgresql &&\
    cargo install diesel_cli --no-default-features --features postgres
COPY . /app
WORKDIR /app
RUN cp .env.docker .env
