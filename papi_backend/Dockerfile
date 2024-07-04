FROM rust:latest

# Install gcc and OpenSSL development libraries
RUN apt-get update && \
    apt-get install -y gcc pkg-config libssl-dev

# Verify installation
RUN rustc --version && cargo --version && gcc --version && openssl version

# Set the working directory inside the container
WORKDIR /papi_backend

# Copy the entire project into the /papi_backend folder inside the container
# Exclude the files and folders specified in .dockerignore
COPY . .

# Build the Rust application
RUN cargo build --release

# Expose the port which the HTTPS server listens on
EXPOSE 8443

# Run the pAPI backend
ENTRYPOINT ["./target/release/personal_api"]