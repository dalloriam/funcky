FROM rust:latest as builder
WORKDIR /usr/src/funcky
COPY . .
RUN cargo install --path funck-server

FROM rust:latest
COPY --from=builder /usr/local/cargo/bin/funck-server /usr/local/bin/funck-server
EXPOSE 3030
CMD ["funck-server"]
