default: install-dev
set windows-powershell := true

# Install ferium to cargo's binary folder
install:
  cargo install --force --path .

# Install ferium to cargo's binary folder, but with faster compilation (offline & debug)
install-dev:
  cargo install --offline --debug --force --path .

# Delete test artefacts
clean-test:
  rm -rf tests/.minecraft \
    tests/md_modpack \
    tests/cf_modpack \
    tests/configs/running \
    tests/configs/profiles/running

##############################
# Cross-compiling to Windows #
##############################

# TIP: Use `wine cmd` and try the `ogj-ferium` command after building.

# Cross compile to Windows (32-bit) and move the binary to `~/.wine/drive_c/windows/ogj-ferium.exe`
[unix]
build-windows32: (cross-windows "i686-pc-windows-gnu")

# Cross compile to Windows (64-bit) and move the binary to `~/.wine/drive_c/windows/ogj-ferium.exe`
[unix]
build-windows64: (cross-windows "x86_64-pc-windows-gnu")

[unix]
[private]
cross-windows target:
  @echo "warning: This just command requires [cross](https://github.com/cross-rs/cross) and [wine](https://www.winehq.org)"
  @echo "warning: This just command is untested on MacOs.\n"

  cross build --release --target {{target}}
  cp target/{{target}}/release/ogj-ferium.exe ~/.wine/drive_c/windows/ogj-ferium.exe
