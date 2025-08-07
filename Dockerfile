# Build and runtime in same image to avoid glibc mismatch
FROM rust:1.88-slim

WORKDIR /app

# Copy files
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src

# Build the app
RUN cargo build --release

# Copy assets
COPY backend/templates ./templates
COPY backend/static ./static
COPY backend/posts ./posts
COPY backend/posts.json ./posts.json

# Optional: safer user
RUN useradd -m bloguser
USER bloguser

WORKDIR /app

EXPOSE 8000

CMD ["./target/release/backend"]
