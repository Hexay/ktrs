# ghcr.io/hexay/ktrs: the release's static musl binaries, no compile. Context: tools/release/docker-context.sh.
#   docker run --rm -v "$PWD:/src" ghcr.io/hexay/ktrs ktlint "**/*.kt"
# distroless/static over scratch: same zero-library runtime, plus /tmp, /etc/passwd and CA certs.
# No ENTRYPOINT: the first argument picks the binary (ktrs, ktlint or ktfmt).
# `ktlint -R <jar>` other than compose-rules hands off to the real ktlint jar, which needs Java: not in this image.
FROM gcr.io/distroless/static-debian12
ARG TARGETARCH
ARG VERSION=dev
LABEL org.opencontainers.image.title="ktrs" \
      org.opencontainers.image.description="Fast Kotlin formatter and linter: drop-in ktfmt and ktlint binaries" \
      org.opencontainers.image.source="https://github.com/Hexay/ktrs" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"
COPY --chmod=0755 linux-${TARGETARCH}/ /usr/local/bin/
WORKDIR /src
CMD ["ktrs", "--help"]
