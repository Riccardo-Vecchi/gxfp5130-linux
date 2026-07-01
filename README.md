# GXFP5130 Linux fingerprint sensor support

[![Status](https://img.shields.io/badge/status-experimental-orange)](#current-status)
[![Tested on Zorin OS 18.1](https://img.shields.io/badge/tested%20on-Zorin%20OS%2018.1-blue)](docs/tested-host.md)
[![Kernel](https://img.shields.io/badge/kernel-7.0.14-informational)](docs/tested-host.md)
[![open-fprintd](https://img.shields.io/badge/fingerprint-open--fprintd-success)](https://github.com/uunicorn/open-fprintd)
[![License](https://img.shields.io/badge/license-GPL--2.0%20%2B%20LGPL--2.1-lightgrey)](#license)

Linux integration for the Goodix `GXFP5130` press fingerprint sensor, validated
on a Huawei MateBook X Pro 2024.

## Table Of Contents

- [Current Status](#current-status)
- [Tested Hardware](#tested-hardware)
- [Why This Repository Exists](#why-this-repository-exists)
- [Upstream Comparison](#upstream-comparison)
- [What Is Included, And Why](#what-is-included-and-why)
- [Install Outline](#install-outline)
- [Debugging](#debugging)
- [Provisioning And PSK](#provisioning-and-psk)
- [Security Notes](#security-notes)
- [Contributing](#contributing)
- [Credits](#credits)
- [License](#license)
- [Current Limitations](#current-limitations)

This repository turns the current GXFP5130 reverse-engineering pieces into a
working Linux fingerprint stack:

```text
GXFP5130 sensor -> gxfp kernel module -> gxfpmoc capture helper
               -> gxfp-openfprintd backend -> open-fprintd
               -> fprintd CLI / GNOME Settings / PAM / GDM
```

## Current Status

- DKMS-managed `gxfp` kernel module for the GXFP5130 eSPI transport.
- Goodix MOC one-shot capture helper using a reprovisioned PSK.
- `open-fprintd` backend visible through `fprintd-list`, `fprintd-enroll`,
  `fprintd-verify`, GNOME Settings, PAM and GDM.
- Enrollment and verification using extracted matcher features, not raw image
  storage.
- Verified behavior: enrolled right index matched; another finger did not.

This is not yet a universal Linux support claim for every GXFP5130 laptop. It is
tested support for the hardware listed in [Tested Hardware](#tested-hardware)
and a practical starting point for nearby Goodix GXFP5130 devices.

## Tested Hardware

| Laptop | Sensor ACPI ID | Firmware | OS | Kernel | Status |
| --- | --- | --- | --- | --- | --- |
| Huawei MateBook X Pro 2024 | `GXFP5130:00` | `GF_GCC_EC_20068` | Zorin OS 18.1 | `7.0.14` | enroll + verify |

Got it working on different hardware? Open a PR adding a row, or open a
compatibility report issue with the details listed in
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## Why This Repository Exists

Switching to Linux and losing fingerprint login was the kind of small
annoyance that turns into a weekend project.

The upstream projects below solve the kernel transport, Goodix MOC protocol,
matching, and fprintd-compatible frontend separately. This repository packages
the missing integration layer needed for a desktop login workflow on the tested
GXFP5130 laptop. Background discussion and early GXFP5130 clues are available in
[`Void755/gxfp_linux_driver` issue #1](https://github.com/Void755/gxfp_linux_driver/issues/1).

## Upstream Comparison

| Project | What it provides | What this repo adds |
| --- | --- | --- |
| [`Void755/gxfp_linux_driver`](https://github.com/Void755/gxfp_linux_driver) | Kernel-side GXFP5130 eSPI transport and `/dev/gxfp` interface. | DKMS install glue, boot integration, service dependency wiring, and docs for the tested desktop stack. |
| [`Void755/gxfpmoc`](https://github.com/Void755/gxfpmoc) | Goodix MOC protocol, TLS/PSK session handling, provisioning, recovery and capture primitives. | A patch adding a one-shot capture helper for daemon use plus mbedTLS compatibility fixes used by this integration. |
| [`AlexDaichendt/sil6250-linux`](https://github.com/AlexDaichendt/sil6250-linux/tree/main) | SIL6250 Linux work, reverse-engineering notes, and an LGPL matcher implementation. | Reuses the matcher crate for GXFP 80x64 captures and tunes enrollment/verification thresholds for this sensor path. |
| [`uunicorn/open-fprintd`](https://github.com/uunicorn/open-fprintd) | fprintd-compatible D-Bus frontend for standalone backend daemons. | A GXFP5130-specific backend daemon that registers with open-fprintd and stores extracted features. |

## What Is Included, And Why

This repository intentionally does not vendor everything; in other words, it
does not copy every upstream source tree into this repo. Some dependencies are
fetched fresh at install time instead.

Included:

- `openfprintd-backend/` - the GXFP5130 backend daemon written for this setup.
- `crates/sil6250/` - the LGPL-2.1 matcher crate needed to build the backend.
- `patches/` - the small patch applied to `Void755/gxfpmoc`.
- `scripts/` - fetch/install helpers, DKMS config and service glue.
- `systemd/` - backend service unit.
- `docs/` - provisioning, debug and tested-host notes.
- `CONTRIBUTING.md` - compatibility-reporting and contribution guidance.
- `UPSTREAMS.md` - pinned upstream revisions and provenance notes.
- `LICENSES-GPL-2.0.txt`, `LICENSES-LGPL-2.1.txt` - license texts for the
  integration code and matcher crate.
- `gxfp-backup/`, `gxfp-reprovision/` - local-only directories for private
  material; their contents are ignored by Git.

Not vendored:

- Full `gxfp_linux_driver` and `gxfpmoc` source trees. The installer fetches
  pinned upstream revisions instead, keeping provenance clear.
- Sensitive material (PSKs, factory blobs, captures, templates) — see
  [Security Notes](#security-notes) for the full list and why.

## Install Outline

Install development dependencies first. On Debian/Ubuntu/Zorin-like systems:

```bash
sudo apt install build-essential dkms linux-headers-$(uname -r) \
  git rsync patch cmake pkg-config libmbedtls-dev python3 python3-setuptools \
  cargo rustc libpam-fprintd
```

Fetch pinned upstreams and apply the local `gxfpmoc` patch:

```bash
./scripts/fetch-upstreams.sh
```

Install the kernel module, helpers, `open-fprintd`, backend daemon and services:

```bash
sudo ./scripts/install.sh
```

Make sure your PSK exists at:

```text
/var/lib/open-fprintd/gxfp/psk-new-raw32.bin
```

If you do not have one yet, see [Provisioning And PSK](#provisioning-and-psk).
Try recovering an existing PSK first when possible. Invasive reprovisioning is
the fallback because it writes persistent sensor state.

Then enroll:

```bash
fprintd-enroll -f right-index-finger "$USER"
```

And verify:

```bash
fprintd-verify -f right-index-finger "$USER"
```

## Debugging

Useful first checks:

```bash
systemctl status open-fprintd.service gxfp-openfprintd.service --no-pager
fprintd-list "$USER"
journalctl -u gxfp-openfprintd.service -u open-fprintd.service -n 80 --no-pager
journalctl -k -b --no-pager | grep -Ei 'gxfp|GXFP5130|finger|goodix'
```

The backend supports opt-in raw capture diagnostics:

```ini
Environment=GXFP_DEBUG_CAPTURE_DIR=/root/gxfp-debug-captures
```

Only enable this temporarily. It saves raw fingerprint images, which are
biometric data. Use a root-only directory and delete captures after analysis.

More detail: [`docs/debugging.md`](docs/debugging.md).

## Provisioning And PSK

On the tested machine, the factory/current PSK did not authenticate and capture
failed with:

```text
SSL - Verification of the message MAC failed
```

Linux capture started working only after invasive reprovisioning with a new PSK.
That may invalidate Windows-side fingerprint enrollment and can require
re-enrollment in Windows.

Provisioning notes live in [`docs/provisioning.md`](docs/provisioning.md).
Read that guide in full before running any provisioning step. Try recovering an
existing PSK first; invasive reprovisioning is destructive and machine-specific.
See [Security Notes](#security-notes) for what must stay private.

## Security Notes

- Do not publish PSKs, factory blobs, trace files, provisioning logs, raw PGM
  captures or enrolled templates.
- `gxfp-backup/` and `gxfp-reprovision/` are present for local convenience, but
  their contents are ignored by Git.
- `GXFP_DEBUG_CAPTURE_DIR` saves biometric raw images. Use it only briefly.
- Fingerprint login usually cannot unlock the GNOME keyring because PAM did not
  receive your password.
- The installer masks stock `fprintd.service` because `open-fprintd` owns the
  same `net.reactivated.Fprint` D-Bus name.

## Contributing

Hardware compatibility reports are especially useful. Please include:

- laptop model and BIOS/firmware version when available
- sensor ACPI ID, for example from `journalctl -k -b | grep -i GXFP`
- kernel version from `uname -a`
- OS/distribution
- whether capture, enroll and verify work
- relevant logs with PSKs, blobs, traces and raw captures removed

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full checklist and safety
rules.

## Credits

This integration stands on top of work from:

- [Void755/gxfp_linux_driver](https://github.com/Void755/gxfp_linux_driver) for
  the GXFP5130 Linux kernel transport.
- [Void755/gxfpmoc](https://github.com/Void755/gxfpmoc) for the Goodix MOC
  userspace protocol, TLS/PSK session handling, provisioning, recovery and
  capture work.
- [AlexDaichendt/sil6250-linux](https://github.com/AlexDaichendt/sil6250-linux/tree/main)
  for the matcher and reverse-engineering notes that made local matching
  practical.
- [uunicorn/open-fprintd](https://github.com/uunicorn/open-fprintd) for the
  standalone backend architecture.

Pinned revisions and license notes: [`UPSTREAMS.md`](UPSTREAMS.md).

## License

Original integration code, scripts, service files and documentation in this
repository are GPL-2.0 only. See [`LICENSES-GPL-2.0.txt`](LICENSES-GPL-2.0.txt).

The vendored matcher crate under `crates/sil6250/` comes from `sil6250-linux`
and is LGPL-2.1. See [`LICENSES-LGPL-2.1.txt`](LICENSES-LGPL-2.1.txt).

## Current Limitations

- Tested on one Huawei MateBook X Pro 2024 system, not a broad device matrix.
- Provisioning remains intentionally manual because it changes persistent sensor
  state.
