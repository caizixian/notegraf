# syntax=docker/dockerfile:1
FROM alpine:3
WORKDIR /notegraf
COPY ./target/x86_64-unknown-linux-musl/release/notegraf-web /notegraf/notegraf-web
COPY ./notegraf-web/dist /notegraf/dist
ENTRYPOINT ["/notegraf/notegraf-web"]
