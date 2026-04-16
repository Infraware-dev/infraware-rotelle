FROM scratch

ARG MUSL_TARGET
COPY rotelle/target/${MUSL_TARGET}/release/rotelle /usr/local/bin/rotelle

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/rotelle"]
