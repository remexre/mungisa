FROM rust:latest

WORKDIR /usr/src/mungisa
COPY . .

RUN cargo install --path /usr/src/mungisa
CMD ["mungisa"]
