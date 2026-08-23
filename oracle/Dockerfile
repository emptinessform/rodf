# rodf 결정론 오라클 — LO 버전·폰트·래스터라이저가 고정된 재현 환경.
#
# Debian bookworm이 LibreOffice(7.4.x)·poppler(pdftoppm)·Noto CJK KR 버전을
# 배포판 수준에서 고정한다. 코퍼스는 컨테이너 안에서 noto 폰트 셋으로
# 생성되므로 호스트 폰트에 의존하지 않는다.
#
# 빌드:  docker build -t rodf-oracle -f oracle/Dockerfile .
# 실행:  docker run --rm -v <repo>:/work rodf-oracle
#        (결과: /work/docs/scoreboard-ci.{md,json})

FROM rust:1.95-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
        libreoffice-writer \
        fonts-noto-cjk \
        poppler-utils \
        python3 \
        python3-pil \
        python3-numpy \
    && rm -rf /var/lib/apt/lists/* \
    && soffice --version && pdftoppm -v 2>&1 | head -1

WORKDIR /work
# Windows 바인드 마운트의 I/O 페널티를 피해 빌드 산출물은 컨테이너 쪽에 둔다.
ENV CARGO_TARGET_DIR=/tmp/target

COPY oracle/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
