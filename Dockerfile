FROM rust:slim-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libsqlite3-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Add musl target
RUN rustup target add x86_64-unknown-linux-musl

# Create a new empty shell project
WORKDIR /usr/src/craftcms
COPY . .

# Create data directory
RUN mkdir -p data

# Build static release version
RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage
FROM scratch

# Copy our static binary
COPY --from=builder /usr/src/craftcms/target/x86_64-unknown-linux-musl/release/craftcms /craftcms

# Copy static assets and templates
COPY --from=builder /usr/src/craftcms/templates /templates
COPY --from=builder /usr/src/craftcms/static /static

# Copy empty data directory
COPY --from=builder /usr/src/craftcms/data /data

# Set the binary as entrypoint
ENTRYPOINT ["/craftcms"]
CMD ["serve"]

# Expose the default port
EXPOSE 8080
