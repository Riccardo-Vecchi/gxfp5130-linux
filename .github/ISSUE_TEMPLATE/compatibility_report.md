---
name: Compatibility report
about: Report whether GXFP5130 works on your laptop
title: "[compat] "
labels: compatibility
assignees: ""
---

## Hardware

- Laptop model:
- BIOS/firmware version:
- Sensor ACPI ID:
- Fingerprint firmware string, if shown by `gxfp`:

## Software

- Distribution:
- Kernel (`uname -a`):
- Secure Boot enabled:

## Results

- `gxfp` kernel module loads: yes/no
- `/dev/gxfp` exists: yes/no
- `gxfp_capture_once` captures: yes/no
- `fprintd-list` shows a device: yes/no
- `fprintd-enroll` works: yes/no
- `fprintd-verify` accepts enrolled finger: yes/no
- `fprintd-verify` rejects another finger: yes/no

## Logs

Paste relevant logs with PSKs, blobs, traces and raw captures removed.

```bash
journalctl -k -b --no-pager | grep -Ei 'gxfp|GXFP5130|finger|goodix'
```

```bash
journalctl -u gxfp-openfprintd.service -u open-fprintd.service -n 120 --no-pager
```

## Notes

