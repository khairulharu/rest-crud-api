# Build Stage
FROM rust:1.84.0 as builder

WORKDIR /app

# accept the build argument
ARG DATABASE_URL

ENV DATABASE_URL=$DATABASE_URL

COPY . .

RUN cargo build --release

# production stage
FROM ubuntu::22.04

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/rest-crud-api .

CMD [ "./rest-crud-api" ]