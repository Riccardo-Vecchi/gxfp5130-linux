# Tested host

The current integration was validated on:

## Huawei MateBook X Pro 2024 — Zorin OS 18.1

- Hardware: Huawei MateBook X Pro 2024
- ACPI device: `GXFP5130:00`
- Firmware string reported by driver: `GF_GCC_EC_20068`
- OS: Zorin OS 18.1, Ubuntu-based
- Kernel: `7.0.14`
- Driver mode: DKMS `gxfp/0.1.0`
- Fingerprint stack: `open-fprintd` backend exposed through `fprintd-*`, PAM,
  GNOME Settings and GDM.

Final validation:

- `fprintd-list "$USER"` exposed one press-type device.
- `fprintd-verify -f right-index-finger "$USER"` matched the enrolled right
  index.
- A different finger did not match.
- Final matcher threshold: SIFT inliers `>= 5`.
- Verification quality gate: `0.25`.
- Enrollment quality gate: `0.52`.

Kernel log notes:

- `gxfp` logs only normal boot/init lines on the tested host.
- `int3472-discrete ... Failed to get GPIO`, `intel-ipu6`, `uvcvideo`, and
  `v4l2loopback` messages observed on the host were related to camera/video
  paths, not to the fingerprint sensor.
- Large volumes of `[UFW BLOCK]` messages may dominate `journalctl -k` and are
  unrelated to this driver.

## Huawei MateBook 14 (Core Ultra) — Ubuntu 26.04 LTS

- Hardware: Huawei MateBook 14 (Core Ultra), model `FLMH-XX`, BIOS `1.22`
- CPU: Intel Core Ultra 7 155H
- ACPI device: `GXFP5130:00`
- Firmware string reported by driver: `GF_GCC_EC_20068`
- OS: Ubuntu 26.04 LTS (codename `resolute`)
- Kernel: `7.0.0-27-generic` (x86_64, PREEMPT_DYNAMIC)
- Driver mode: DKMS `gxfp/0.1.0`
- Fingerprint stack: `open-fprintd` backend exposed through `fprintd-*`, PAM,
  GNOME Settings (v50, Wayland) and GDM.

Install notes:

- Required removing `LLVM=1` from the DKMS make flags because the Ubuntu 26.04
  7.0 kernel uses GCC-specific compile flags that Clang does not support.
- Required a `cd` to the `open-fprintd` source directory before running
  `setup.py install` for setuptools to locate the `openfprintd` package.
- Required adding a D-Bus policy file to allow the `gxfp-openfprintd` backend
  daemon to own the `io.github.uunicorn.Fprint` bus name.

Final validation:

- `fprintd-list "$USER"` exposed one device at
  `/net/reactivated/Fprint/Device/0`.
- `fprintd-enroll -f right-thumb "$USER"` completed 5 stages
  (`enroll-completed`) without quality or diversity rejections.
- `fprintd-verify -f right-thumb "$USER"` returned `verify-match`.
- `fprintd-enroll -f right-index-finger "$USER"` completed 5 stages
  (`enroll-completed`) without quality or diversity rejections.
- `fprintd-verify -f right-index-finger "$USER"` returned `verify-match`.
- `fprintd-enroll -f right-middle-finger "$USER"` completed 5 stages
  (`enroll-completed`) without quality or diversity rejections.
- `fprintd-verify -f right-middle-finger "$USER"` returned `verify-match`.

Kernel log notes:

- `gxfp: loading out-of-tree module taints kernel.`
- `gxfp GXFP5130:00: INIT: FW='GF_GCC_EC_20068'`
- `gxfp GXFP5130:00: IRQ: trigger type unknown; defaulting to LEVEL_HIGH`
- No errors or warnings beyond the expected IRQ trigger type note.
