FROM scratch

ARG MUSL_TARGET
COPY rotelle/target/${MUSL_TARGET}/release/rotelle /usr/local/bin/rotelle

EXPOSE 8080

# Run as unprivileged user (UID 65534 = nobody).
# scratch has no /etc/passwd, so we set a numeric UID/GID directly.
USER 65534:65534

ENTRYPOINT ["/usr/local/bin/rotelle"]
