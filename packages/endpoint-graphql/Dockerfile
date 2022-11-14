FROM rust:1.65.0-alpine AS builder

COPY . .

RUN cargo build -p endpoint-graphql --bin endpoint-graphql --release

FROM alpine:3.16.3

RUN addgroup -S wonderland && adduser -S whiterabbit -G wonderland

WORKDIR /app

RUN chown -R whiterabbit:wonderland /app

COPY --from=builder target/release endpoint-graphql /app/

USER whiterabbit

ENTRYPOINT ["/app/endpoint-graphql"]
