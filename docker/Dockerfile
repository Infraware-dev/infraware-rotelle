FROM scratch

ARG MUSL_TARGET=aarch64-unknown-linux-musl
COPY rotelle/target/${MUSL_TARGET}/release/rotelle /usr/local/bin/rotelle

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/rotelle"]
