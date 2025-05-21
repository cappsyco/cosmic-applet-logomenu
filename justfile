name1 := 'cosmic-applet-logomenu'
name2 := 'cosmic-logomenu-settings'
export APPID1 := 'co.uk.cappsy.CosmicAppletLogoMenu'
export APPID2 := 'co.uk.cappsy.CosmicLogoMenuSettings'

rpm_arch := arch()
deb_arch := if rpm_arch == "x86_64" { "amd64" } else { rpm_arch }
version := `sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`

rootdir := ''
prefix := '/usr'
flatpak-prefix := '/app'

base-dir := absolute_path(clean(rootdir / prefix))
flatpak-base-dir := absolute_path(clean(rootdir / flatpak-prefix))

export INSTALL_DIR := base-dir / 'share'

bin-src1 := 'target' / 'release' / name1
bin-src2 := 'target' / 'release' / name2
bin-dst1 := base-dir / 'bin' / name1
bin-dst2 := base-dir / 'bin' / name2
flatpak-bin-dst1 := flatpak-base-dir / 'bin' / name1
flatpak-bin-dst2 := flatpak-base-dir / 'bin' / name2

desktop1 := APPID1 + '.desktop'
desktop2 := APPID2 + '.desktop'
desktop-src1 := 'res' / desktop1
desktop-src2 := 'res' / desktop2
desktop-dst1 := clean(rootdir / prefix) / 'share' / 'applications' / desktop1
desktop-dst2 := clean(rootdir / prefix) / 'share' / 'applications' / desktop2

metainfo1 := APPID1 + '.metainfo.xml'
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
    install -Dm0644 "{{icons-src}}/scalable/apps/{{APPID1}}.svg" "{{icons-dst}}/scalable/apps/{{APPID1}}.svg"; \
    install -Dm0644 "{{icons-src}}/scalable/apps/{{APPID2}}.svg" "{{icons-dst}}/scalable/apps/{{APPID2}}.svg"; \

# Uninstalls installed files
uninstall:
    rm {{bin-dst1}}
    rm {{bin-dst2}}
    rm {{desktop-dst1}}
    rm {{desktop-dst2}}
    rm {{metainfo-dst1}}
    rm {{metainfo-dst2}}
    rm "{{icons-dst}}/scalable/apps/{{APPID1}}.svg"; \
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

# Deb install
debname1 := name1+'_'+version+'_'+architecture
debname2 := name2+'_'+version+'_'+architecture
debdir1 := debname1 / 'DEBIAN'
debdir2 := debname2 / 'DEBIAN'
debcontrol1 := debdir1 / 'control'
debcontrol2 := debdir2 / 'control'

deb:
    strip {{bin-src1}}
    install -D {{bin-src1}} {{debname1}}{{bin-dst1}}
    install -D {{desktop-src1}} {{debname1}}{{desktop-dst1}}
    install -D "{{icons-src}}/scalable/apps/{{APPID1}}.svg" "{{debname1}}{{icons-dst}}/apps/{{APPID1}}.svg"; \
    mkdir -p {{debdir1}}
    echo "Package: {{name1}}" > {{debcontrol1}}
    echo "Version: {{version}}" >> {{debcontrol1}}
    echo "Architecture: {{architecture}}" >> {{debcontrol1}}
    echo "Maintainer: {{dev_name}} <{{email}}>" >> {{debcontrol1}}
    echo "Description: {{summary}}" >> {{debcontrol1}}
    dpkg-deb --build --root-owner-group {{debname1}}
    rm -Rf {{debname1}}/

    strip {{bin-src2}}
    install -D {{bin-src2}} {{debname2}}{{bin-dst2}}
    install -D {{desktop-src2}} {{debname2}}{{desktop-dst2}}
    install -D "{{icons-src}}/scalable/apps/{{APPID2}}.svg" "{{debname1}}{{icons-dst}}/apps/{{APPID2}}.svg"; \
    mkdir -p {{debdir2}}
    echo "Package: {{name2}}" > {{debcontrol2}}
    echo "Version: {{version}}" >> {{debcontrol2}}
    echo "Architecture: {{architecture}}" >> {{debcontrol2}}
    echo "Maintainer: {{dev_name}} <{{email}}>" >> {{debcontrol2}}
    echo "Description: {{summary}}" >> {{debcontrol2}}
    dpkg-deb --build --root-owner-group {{debname2}}
    rm -Rf {{debname2}}/




# RPM install

rpmarch := arch()
rpmname := name + '-' + version + '-1.' + rpmarch
rpmdir := rpmname / 'BUILDROOT'
rpminstall := rpmdir / prefix
rpm_bin_dst := rpminstall / 'bin' / name
rpm_desktop_dst := rpminstall / 'share' / 'applications' / desktop
rpm_metainfo_dst := rpminstall / 'share' / 'metainfo' / metainfo
rpm_icons_dst := rpminstall / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps'

rpm:
    strip {{bin-src}}
    install -D {{bin-src}} {{rpm_bin_dst}}
    install -D {{desktop-src}} {{rpm_desktop_dst}}
    install -D {{metainfo-src}} {{rpm_metainfo_dst}}
    for svg in {{icons-src}}/apps/*.svg; do \
        install -D "$svg" "{{rpm_icons_dst}}/$(basename $svg)"; \
    done

    mkdir -p {{rpmname}}
    echo "Name: {{name}}" > {{rpmname}}/spec.spec
    echo "Version: {{version}}" >> {{rpmname}}/spec.spec
    echo "Release: 1%{?dist}" >> {{rpmname}}/spec.spec
    echo "Summary: {{summary}}" >> {{rpmname}}/spec.spec
    echo "" >> {{rpmname}}/spec.spec
    echo "License: GPLv3" >> {{rpmname}}/spec.spec
    echo "Group: Applications/Utilities" >> {{rpmname}}/spec.spec
    echo "%description" >> {{rpmname}}/spec.spec
    echo "{{summary}}" >> {{rpmname}}/spec.spec
    echo "" >> {{rpmname}}/spec.spec
    echo "%files" >> {{rpmname}}/spec.spec
    echo "%defattr(-,root,root,-)" >> {{rpmname}}/spec.spec
    echo "{{prefix}}/bin/{{name}}" >> {{rpmname}}/spec.spec
    echo "{{prefix}}/share/applications/{{desktop}}" >> {{rpmname}}/spec.spec
    echo "{{prefix}}/share/metainfo/{{metainfo}}" >> {{rpmname}}/spec.spec
    echo "{{prefix}}/share/icons/hicolor/scalable/apps/*.svg" >> {{rpmname}}/spec.spec

    rpmbuild -bb --buildroot="$(pwd)/{{rpmdir}}" {{rpmname}}/spec.spec \
        --define "_rpmdir $(pwd)" \
        --define "_topdir $(pwd)/{{rpmname}}" \
        --define "_buildrootdir $(pwd)/{{rpmdir}}"

    rm -rf {{rpmname}} {{rpmdir}}
    mv x86_64/* .
    rmdir x86_64
