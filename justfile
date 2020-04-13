build:
    cargo build

docker:
  docker build -t dalloriam/funcky -f docker/Server.dockerfile .

validate:
    cargo check
    cargo clippy
