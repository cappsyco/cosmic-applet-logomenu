name := 'cosmic-applet-logomenu'
name2 := 'cosmic-logomenu-settings'
export APPID := 'co.uk.cappsy.CosmicAppletLogoMenu'
export APPID2 := 'co.uk.cappsy.CosmicLogoMenuSettings'

rootdir := ''
prefix := '/usr'
flatpak-prefix := '/app'

base-dir := absolute_path(clean(rootdir / prefix))
flatpak-base-dir := absolute_path(clean(rootdir / flatpak-prefix))

export INSTALL_DIR := base-dir / 'share'

bin-src1 := 'target' / 'release' / name
bin-src2 := 'target' / 'release' / name2
bin-dst1 := base-dir / 'bin' / name
bin-dst2 := base-dir / 'bin' / name2
flatpak-bin-dst1 := flatpak-base-dir / 'bin' / name
flatpak-bin-dst2 := flatpak-base-dir / 'bin' / name2

desktop1 := APPID + '.desktop'
desktop2 := APPID2 + '.desktop'
desktop-src1 := 'res' / desktop1
desktop-src2 := 'res' / desktop2
desktop-dst1 := clean(rootdir / prefix) / 'share' / 'applications' / desktop1
desktop-dst2 := clean(rootdir / prefix) / 'share' / 'applications' / desktop2

metainfo1 := APPID + '.metainfo.xml'
metainfo2 := APPID2 + '.metainfo.xml'
metainfo-src1 := 'res' / metainfo1
metainfo-src2 := 'res' / metainfo2
metainfo-dst1 := clean(rootdir / prefix) / 'share' / 'metainfo' / metainfo1
metainfo-dst2 := clean(rootdir / prefix) / 'share' / 'metainfo' / metainfo2

icons-src := 'res' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Compiles with debug profile
build-debug *args:
    (cd applet && cargo build {{args}})
    (cd settings && cargo build {{args}})

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

dev *args:
    cargo fmt
    just run {{args}}

# Run with debug logs
run *args:
    env RUST_LOG=cosmic_tasks=info RUST_BACKTRACE=full cargo run --release {{args}}

# Installs files
install:
    install -Dm0755 {{bin-src1}} {{bin-dst1}}
    install -Dm0755 {{bin-src2}} {{bin-dst2}}
    install -Dm0644 {{desktop-src1}} {{desktop-dst1}}
    install -Dm0644 {{desktop-src2}} {{desktop-dst2}}
    install -Dm0644 {{metainfo-src1}} {{metainfo-dst1}}
    install -Dm0644 {{metainfo-src2}} {{metainfo-dst2}}
    install -Dm0644 "{{icons-src}}/scalable/apps/{{APPID}}.svg" "{{icons-dst}}/scalable/apps/{{APPID}}.svg"; \
    install -Dm0644 "{{icons-src}}/scalable/apps/{{APPID2}}.svg" "{{icons-dst}}/scalable/apps/{{APPID2}}.svg"; \

# Installs files
flatpak:
    install -Dm0755 {{bin-src1}} {{flatpak-bin-dst2}}
    install -Dm0755 {{bin-src2}} {{flatpak-bin-dst2}}
    install -Dm0644 {{desktop-src1}} {{desktop-dst1}}
    install -Dm0644 {{desktop-src2}} {{desktop-dst2}}
    install -Dm0644 {{metainfo-src1}} {{metainfo-dst1}}
    install -Dm0644 {{metainfo-src2}} {{metainfo-dst2}}
    install -Dm0644 "{{icons-src}}/scalable/apps/{{APPID}}.svg" "{{icons-dst}}/scalable/apps/{{APPID}}.svg"; \
    install -Dm0644 "{{icons-src}}/scalable/apps/{{APPID2}}.svg" "{{icons-dst}}/scalable/apps/{{APPID2}}.svg"; \

# Uninstalls installed files
uninstall:
    rm {{bin-dst1}}
    rm {{bin-dst2}}
    rm {{desktop-dst1}}
    rm {{desktop-dst2}}
    rm {{metainfo-dst1}}
    rm {{metainfo-dst2}}
    rm "{{icons-dst}}/scalable/apps/{{APPID}}.svg"; \
    rm "{{icons-dst}}/scalable/apps/{{APPID2}}.svg"; \

# Vendor dependencies locally
vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$(date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

# Extracts vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar
