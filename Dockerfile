FROM rust:slim-bullseye as builder

# Install SQLite dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/craftcms
COPY . .

# Create data directory
RUN mkdir -p data

# Build release version
RUN cargo build --release

# Final stage
FROM scratch

# Copy our binary
COPY --from=builder /usr/src/craftcms/target/release/craftcms /craftcms

# Copy static assets and templates
COPY --from=builder /usr/src/craftcms/templates /templates
# COPY --from=builder /usr/src/craftcms/assets /assets
COPY --from=builder /usr/src/craftcms/static /static

# Copy empty data directory
COPY --from=builder /usr/src/craftcms/data /data

# Set the binary as entrypoint
ENTRYPOINT ["/craftcms"]
CMD ["serve"]

# Expose the default port
EXPOSE 8080
