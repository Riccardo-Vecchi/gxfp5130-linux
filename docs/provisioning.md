# Provisioning and PSK

GXFP MOC communication uses a PSK. If the factory PSK cannot authenticate,
capture will fail with errors like:

```text
SSL - Verification of the message MAC failed
```

The tested machine required invasive reprovisioning with a new PSK before Linux
captures worked.

## Risk

Reprovisioning writes new device state. It can invalidate existing Windows
fingerprint enrollment and may require re-enrolling fingerprints in Windows.
Keep backups of factory blobs before writing anything.

## Private files

Keep all generated PSKs, `bb010002`/`bb010003` blobs, logs, traces, and raw
captures under `gxfp-backup/` or `gxfp-reprovision/`. These paths are ignored by
Git in this repository.

## Tested PSK location

The backend service expects:

```text
/var/lib/open-fprintd/gxfp/psk-new-raw32.bin
```

The file should be root-only:

```bash
install -d -m 0700 /var/lib/open-fprintd/gxfp
install -m 0600 psk-new-raw32.bin /var/lib/open-fprintd/gxfp/psk-new-raw32.bin
```

This repository intentionally does not include a PSK or provisioning blob.
