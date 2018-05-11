FROM rust:latest

WORKDIR /usr/src/mungisa
COPY . .

RUN cargo install
CMD ["mungisa"]
