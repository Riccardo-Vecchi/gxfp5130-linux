# Tested host

The current integration was validated on:

- Hardware: Huawei MateBook X Pro 2024
- ACPI device: `GXFP5130:00`
- Firmware string reported by driver: `GF_GCC_EC_20068`
- OS: Zorin OS
- Kernel: `7.0.14-x64v3-xanmod1`
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
