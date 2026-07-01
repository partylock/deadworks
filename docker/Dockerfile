# syntax=docker/dockerfile:1.4
# App layer on top of partylock/gameserver-runtime (Proton/Steam/.NET baked separately).
#   docker build -f docker/Dockerfile \
#     --build-arg RUNTIME_IMAGE=partylock/gameserver-runtime:latest \
#     --build-context extra-plugins=plugins \
#     -t partylock/gameserver:latest .

ARG RUNTIME_IMAGE=partylock/gameserver-runtime:latest

FROM ubuntu:24.04 AS native-builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
 wget gnupg ca-certificates software-properties-common \
 cmake ninja-build unzip git python3 \
 && wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | tee /etc/apt/trusted.gpg.d/apt.llvm.org.asc \
 && echo "deb http://apt.llvm.org/noble/ llvm-toolchain-noble-20 main" > /etc/apt/sources.list.d/llvm.list \
 && apt-get update && apt-get install -y --no-install-recommends \
 clang-20 lld-20 llvm-20 \
 && rm -rf /var/lib/apt/lists/*

RUN printf '#!/bin/sh\nexec /usr/bin/clang-20 --driver-mode=cl "$@"\n' > /usr/local/bin/clang-cl && chmod +x /usr/local/bin/clang-cl \
 && printf '#!/bin/sh\nexec /usr/bin/lld-20 -flavor link "$@"\n' > /usr/local/bin/lld-link && chmod +x /usr/local/bin/lld-link \
 && ln -sf /usr/bin/llvm-lib-20 /usr/local/bin/llvm-lib \
 && ln -sf /usr/bin/llvm-rc-20 /usr/local/bin/llvm-rc

ARG XWIN_VERSION=0.6.5
RUN wget -qO /tmp/xwin.tar.gz \
 "https://github.com/Jake-Shadle/xwin/releases/download/${XWIN_VERSION}/xwin-${XWIN_VERSION}-x86_64-unknown-linux-musl.tar.gz" && \
 tar xzf /tmp/xwin.tar.gz -C /tmp && \
 mv /tmp/xwin-*/xwin /usr/local/bin/ && \
 rm -rf /tmp/xwin*

RUN xwin --accept-license splat --output /xwin

ARG DOTNET_VERSION=10.0.0
RUN wget -qO /tmp/nethost.nupkg \
 "https://www.nuget.org/api/v2/package/Microsoft.NETCore.App.Host.win-x64/${DOTNET_VERSION}" && \
 mkdir -p /nethost && \
 unzip -o /tmp/nethost.nupkg -d /tmp/nethost-pkg && \
 cp /tmp/nethost-pkg/runtimes/win-x64/native/nethost.h /nethost/ && \
 cp /tmp/nethost-pkg/runtimes/win-x64/native/coreclr_delegates.h /nethost/ && \
 cp /tmp/nethost-pkg/runtimes/win-x64/native/hostfxr.h /nethost/ && \
 cp /tmp/nethost-pkg/runtimes/win-x64/native/libnethost.lib /nethost/ && \
 rm -rf /tmp/nethost*

RUN wget -qO /tmp/protobuf.tar.gz \
 "https://github.com/protocolbuffers/protobuf/archive/refs/tags/v3.21.8.tar.gz" && \
 tar xzf /tmp/protobuf.tar.gz -C / && \
 mv /protobuf-3.21.8 /protobuf-src && \
 rm /tmp/protobuf.tar.gz

COPY docker/toolchain-clangcl.cmake /toolchain-clangcl.cmake

RUN cmake -G Ninja \
 -S /protobuf-src/cmake \
 -B /protobuf-build \
 -DCMAKE_TOOLCHAIN_FILE=/toolchain-clangcl.cmake \
 -DCMAKE_BUILD_TYPE=Release \
 -DCMAKE_POLICY_DEFAULT_CMP0091=NEW \
 -DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded \
 -Dprotobuf_BUILD_TESTS=OFF \
 -Dprotobuf_MSVC_STATIC_RUNTIME=ON && \
 cmake --build /protobuf-build --config Release

WORKDIR /build
COPY sourcesdk sourcesdk
COPY vendor vendor
COPY protobuf protobuf
COPY config config
COPY docker/build-native.sh docker/build-native.sh
COPY deadworks deadworks

RUN chmod +x docker/build-native.sh && \
 ./docker/build-native.sh

FROM busybox AS extra-plugins
WORKDIR /extra

FROM mcr.microsoft.com/dotnet/sdk:10.0 AS managed-builder

WORKDIR /build
COPY managed managed
COPY game_exported game_exported
COPY docker/plugin-assembly-paths.targets docker/plugin-assembly-paths.targets

RUN --mount=from=extra-plugins,target=/extra \
 mkdir -p extra-plugins && \
 cp -r /extra/. extra-plugins/ 2>/dev/null || true

RUN dotnet publish managed/DeadworksManaged.csproj \
 -c Release -o /artifacts/managed --no-self-contained

RUN mkdir -p /artifacts/managed/plugins && \
 if find extra-plugins -name '*.csproj' -not -path '*/.*' -not -path '*/_backup/*' -not -name '*.Tests.csproj' 2>/dev/null | grep -q .; then \
 cp docker/plugin-assembly-paths.targets extra-plugins/Directory.Build.targets && \
 find extra-plugins -name '*.csproj' -not -path '*/.*' -not -path '*/_backup/*' -not -name '*.Tests.csproj' -print0 | \
 xargs -0 -r -I {} dotnet publish {} -c Release -o /artifacts/managed/plugins --no-self-contained; \
 fi

ARG RUNTIME_IMAGE
FROM ${RUNTIME_IMAGE} AS runtime

COPY --from=native-builder /build/out/deadworks.exe /opt/deadworks/game/bin/win64/
COPY --from=native-builder /build/config/deadworks_mem.jsonc /opt/deadworks/game/citadel/cfg/
COPY --from=managed-builder /artifacts/managed /opt/deadworks/game/bin/win64/managed/
