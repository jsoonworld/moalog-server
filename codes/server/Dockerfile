# ============================================================
# Stage 1: Planner — 의존성 정보 추출
# ============================================================
FROM rust:1.83-bookworm AS planner

RUN cargo install cargo-chef --locked
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ============================================================
# Stage 2: Builder — 의존성 캐시 빌드 + 소스 빌드
# ============================================================
FROM rust:1.83-bookworm AS builder

RUN cargo install cargo-chef --locked
WORKDIR /app

# 의존성만 먼저 빌드 (캐시 레이어)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# 소스 복사 후 실제 빌드
COPY . .
RUN cargo build --release && strip target/release/server

# ============================================================
# Stage 3: Runtime — 최소 런타임 이미지
# ============================================================
FROM debian:bookworm-slim AS runtime

# 런타임 의존성 설치
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 보안: non-root 사용자
RUN groupadd --gid 1001 appuser && \
    useradd --uid 1001 --gid appuser --create-home appuser

WORKDIR /app

# 바이너리 복사
COPY --from=builder /app/target/release/server /app/server

# 폰트 복사 (PDF 생성용)
COPY fonts/ /app/fonts/

# 소유권 설정
RUN chown -R appuser:appuser /app

USER appuser

# 환경변수 기본값
ENV SERVER_PORT=8080
ENV PDF_FONT_DIR=/app/fonts
ENV PDF_FONT_FAMILY=NanumGothic
ENV RUST_LOG=info

EXPOSE 8080

# 헬스체크 — slim 이미지에는 curl이 없으므로 /dev/tcp 사용
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD bash -c 'exec 3<>/dev/tcp/localhost/8080 && echo -e "GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n" >&3 && cat <&3 | grep -q healthy'

ENTRYPOINT ["/app/server"]
