# rodf 결정론 오라클 — LO 버전·폰트·래스터라이저가 고정된 재현 환경.
#
# LibreOffice는 배포판 패키지(bookworm=7.4, 2022 — 구식 Type1 서브세터·구
# 한글 동작)가 아니라 TDF 아카이브의 버전 고정 데비안 패키지를 쓴다.
# 26.2.1.2 = 현재 개발 호스트(Windows) LO와 동일 버전 — 프로파일 간 비교
# 가능성 확보. 폰트는 fonts-noto-cjk(OFL), 코퍼스는 컨테이너 안에서 noto
# 폰트 셋으로 생성되므로 호스트 폰트에 의존하지 않는다.
#
# 빌드:  docker build -t rodf-oracle -f oracle/Dockerfile .
# 실행:  docker run --rm -v <repo>:/work -v rodf-oracle-target:/tmp/target rodf-oracle
#        (결과: /work/docs/scoreboard-ci.{md,json})

FROM rust:1.95-slim-bookworm

ARG LO_VERSION=26.2.1.2
RUN apt-get update && apt-get install -y --no-install-recommends \
        curl ca-certificates \
        fonts-noto-cjk \
        poppler-utils \
        python3 \
        python3-pil \
        python3-numpy \
        libxinerama1 libnss3 libdbus-1-3 libcairo2 libcups2 libx11-xcb1 \
        libxml2 libglib2.0-0 libsm6 libice6 libxrender1 libxext6 libxrandr2 \
    && rm -rf /var/lib/apt/lists/* \
    && curl -fsSL "https://downloadarchive.documentfoundation.org/libreoffice/old/${LO_VERSION}/deb/x86_64/LibreOffice_${LO_VERSION}_Linux_x86-64_deb.tar.gz" \
        -o /tmp/lo.tar.gz \
    && tar -xzf /tmp/lo.tar.gz -C /tmp \
    && dpkg -i /tmp/LibreOffice_*/DEBS/*.deb \
    && rm -rf /tmp/lo.tar.gz /tmp/LibreOffice_* \
    && ln -s /opt/libreoffice*/program/soffice /usr/local/bin/soffice \
    && soffice --version && pdftoppm -v 2>&1 | head -1

WORKDIR /work
# Windows 바인드 마운트의 I/O 페널티를 피해 빌드 산출물은 컨테이너 쪽에 둔다.
ENV CARGO_TARGET_DIR=/tmp/target

COPY oracle/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
