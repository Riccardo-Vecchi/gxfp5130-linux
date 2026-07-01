# Upstreams and Credits

This project glues together several independent reverse-engineering efforts.

## Core upstream work

- `Void755/gxfp_linux_driver`
  - Repository: https://github.com/Void755/gxfp_linux_driver
  - Role: Linux kernel eSPI transport and `/dev/gxfp` userspace interface for
    `GXFP5130`.
  - Tested local revision: `7bd8354`.

- `Void755/gxfpmoc`
  - Repository: https://github.com/Void755/gxfpmoc
  - Role: userspace Goodix MOC protocol, TLS/PSK session handling, provisioning,
    recovery, and capture primitives.
  - Tested local revision: `91995aa`.
  - This repository carries a patch in `patches/` adding the one-shot capture
    helper used by the open-fprintd backend and compatibility fixes for the
    mbedTLS API available on the tested host.

- `AlexDaichendt/sil6250-linux`
  - Repository: https://github.com/AlexDaichendt/sil6250-linux
  - Role: clean-room minutiae/SIFT-style matcher reused here as the local
    matcher for 80x64 GXFP captures.
  - Tested local revision: `d3fe97b`.
  - License: LGPL-2.1, included in `LICENSES-LGPL-2.1.txt`.

- `uunicorn/open-fprintd`
  - Repository: https://github.com/uunicorn/open-fprintd
  - Role: fprintd-compatible D-Bus frontend that allows standalone backend
    daemons.
  - Tested local revision: `b707373`.
  - License: GPL-2.0, included in `LICENSES-GPL-2.0.txt`.

## Local integration work

The `openfprintd-backend/` daemon, installer glue, service files, debugging
workflow, and the GXFP-specific enrollment/verification tuning were built during
testing on a Huawei MateBook X Pro 2024 with ACPI device `GXFP5130:00`.

## License note

Some upstream repositories did not expose a complete top-level license file at
the time this integration was prepared. For that reason this repository does not
vendor their full source trees. The installer fetches them from their canonical
repositories and applies the small patch carried here.
