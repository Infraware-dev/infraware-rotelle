FROM scratch

COPY failer/target/aarch64-unknown-linux-musl/release/failer /usr/local/bin/failer

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/failer"]
