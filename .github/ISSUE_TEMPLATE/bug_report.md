---
name: Bug report
about: Report a failure in install, capture, enrollment, or verification
title: "[bug] "
labels: bug
assignees: ""
---

## Problem

What failed?

## Exact command

```bash

```

## Output

```text

```

## Environment

- Laptop model:
- Sensor ACPI ID:
- Distribution:
- Kernel (`uname -a`):

## Logs

Remove PSKs, blobs, traces, raw captures, and enrolled templates before posting.

```bash
journalctl -k -b --no-pager | grep -Ei 'gxfp|GXFP5130|finger|goodix'
```

```bash
journalctl -u gxfp-openfprintd.service -u open-fprintd.service -n 120 --no-pager
```
