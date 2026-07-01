# GXFP5130 Linux fingerprint support

Experimental Linux support for the Goodix `GXFP5130` press fingerprint sensor
found in at least one Huawei MateBook X Pro 2024 configuration.

This repository packages the integration we validated on:

- laptop: Huawei MateBook X Pro 2024
- ACPI device: `GXFP5130:00`
- OS: Zorin OS
- kernel tested: `7.0.14-x64v3-xanmod1`
- user-facing stack: `open-fprintd` + PAM/GDM through `fprintd-*`

It is not yet a universal Linux driver claim. Treat it as tested support for
this hardware class and a starting point for nearby Goodix GXFP5130 devices.

## What works on the tested machine

- DKMS-managed `gxfp` kernel module for the GXFP5130 eSPI transport.
- Goodix MOC capture helper using a reprovisioned PSK.
- `open-fprintd` backend daemon exposing the sensor to `fprintd-list`,
  `fprintd-enroll`, `fprintd-verify`, GNOME Settings, PAM and GDM.
- Enrollment and verification with extracted minutiae, not raw-image storage.
- Verified behavior: enrolled right index matched; another finger did not.

## Repository layout

- `openfprintd-backend/` - Rust backend daemon for `open-fprintd`.
- `crates/sil6250/` - LGPL-2.1 matcher crate reused from `sil6250-linux`.
- `patches/` - patch applied to `Void755/gxfpmoc` for this integration.
- `scripts/` - fetch/install helpers and DKMS/service glue.
- `systemd/` - backend service unit.
- `docs/` - debugging and provisioning notes.
- `gxfp-backup/`, `gxfp-reprovision/` - local-only private material directories.

## Install outline

Install development dependencies first. On Debian/Ubuntu/Zorin-like systems this
means roughly:

```bash
sudo apt install build-essential dkms linux-headers-$(uname -r) \
  git rsync patch cmake pkg-config libmbedtls-dev python3 python3-setuptools \
  cargo rustc libpam-fprintd
```

Fetch pinned upstreams and apply the local `gxfpmoc` patch:

```bash
./scripts/fetch-upstreams.sh
```

Install kernel module, helpers, `open-fprintd`, backend daemon and services:

```bash
sudo ./scripts/install.sh
```

Make sure your PSK exists at:

```text
/var/lib/open-fprintd/gxfp/psk-new-raw32.bin
```

Then enroll:

```bash
fprintd-enroll -f right-index-finger "$USER"
```

And verify:

```bash
fprintd-verify -f right-index-finger "$USER"
```

## Security notes

- Do not publish PSKs, factory blobs, trace files, logs from provisioning, or
  raw PGM captures.
- `GXFP_DEBUG_CAPTURE_DIR` intentionally saves raw fingerprint images. Use it
  only temporarily with a root-only directory and delete captures afterward.
- Fingerprint login usually cannot unlock the GNOME keyring because PAM did not
  receive your password.
- The installer masks stock `fprintd.service` because `open-fprintd` owns the
  same `net.reactivated.Fprint` D-Bus name.

## Credits

This integration would not exist without:

- Void755 for `gxfp_linux_driver` and `gxfpmoc`.
- The `gxfp_linux_driver` issue discussion that documented the GXFP5130 path.
- AlexDaichendt for `sil6250-linux`, whose matcher and reverse-engineering notes
  made the local matching path practical.
- uunicorn for `open-fprintd`, which makes out-of-tree backends possible without
  forking the whole fprintd/libfprint stack.

See `UPSTREAMS.md` for pinned revisions and license notes.

## Current limitations

- Tested on one Huawei MateBook X Pro 2024 system, not a broad device matrix.
- Some provisioning paths are intentionally documented rather than automated
  because they are invasive.
- Upstream `gxfp_linux_driver` and `gxfpmoc` are fetched from their original
  repositories instead of vendored here because complete top-level licensing was
  not visible when this package was prepared.
