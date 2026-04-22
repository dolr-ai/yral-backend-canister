FROM ghcr.io/dfinity/icp-dev-env:latest

WORKDIR /app

COPY . .

RUN apt-get install wget --no-install-recommends

RUN cargo install candid-extractor --locked

RUN dfx extension install sns --version 0.4.7

CMD ["dfx", "start", "--host", "0.0.0.0:4943", "--artificial-delay", "0"]
