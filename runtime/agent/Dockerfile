FROM docker.io/library/rust@sha256:8a2a102d089fbd151137b9473c01b1ed4519a64fb9c032a03609b815701615c8 AS build
WORKDIR /build
COPY native/Cargo.toml native/Cargo.lock ./native/
COPY embed.rs model.json ./
RUN cargo install --locked --path native --bin mirket-embed --root /out

FROM gcr.io/distroless/cc-debian12@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
COPY --from=build /out/bin/mirket-embed /opt/agent/mirket-embed
COPY model /opt/models
COPY native-libs /opt/native
COPY model.json native.json NOTICE.md /opt/agent/
ENV OMP_NUM_THREADS=2 TOKENIZERS_PARALLELISM=false ORT_DISABLE_TELEMETRY=1
USER 1000:1000
ENTRYPOINT ["/opt/agent/mirket-embed"]
