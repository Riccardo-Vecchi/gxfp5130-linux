# Provisioning and PSK

GXFP MOC communication uses a PSK. If the factory PSK cannot authenticate,
capture will fail with errors like:

```text
SSL - Verification of the message MAC failed
```

The tested machine required invasive reprovisioning with a new PSK before Linux
captures worked.

Read this whole guide before running commands. Provisioning is not part of the
normal install flow: it writes persistent sensor state, and the correct path
depends on what key material is already available on your machine.

## Risk

Reprovisioning writes new device state. It can invalidate existing Windows
fingerprint enrollment and may require re-enrolling fingerprints in Windows.
Keep backups of factory blobs before writing anything.

Prefer this order:

1. Back up the current sensor blobs.
2. Try an existing PSK, including one recovered from the Windows Goodix cache if
   available.
3. Only if that PSK fails, build and upload a new `bb010002` provisioning blob.

## Private files

Keep all generated PSKs, `bb010002`/`bb010003` blobs, logs, traces, and raw
captures under `gxfp-backup/` or `gxfp-reprovision/`. These paths are ignored by
Git in this repository.

Never publish these files in issues, pull requests, screenshots or logs.

## Prerequisites

Run the normal install steps first so the kernel module and helper tools are
available:

```bash
./scripts/fetch-upstreams.sh
sudo ./scripts/install.sh
```

The commands below assume you are in the repository root.

Stop the fingerprint services while testing keys or writing provisioning state:

```bash
sudo systemctl stop gxfp-openfprintd.service open-fprintd.service
```

## Step 1: back up current blobs

Create local private directories:

```bash
mkdir -p gxfp-backup gxfp-reprovision
chmod 0700 gxfp-backup gxfp-reprovision
```

Dump the current provisioning blobs before changing anything:

```bash
sudo gxfp_psk_tool --dump-bb010002 gxfp-backup/bb010002-before.bin
sudo gxfp_psk_tool --dump-bb010003 gxfp-backup/bb010003-before.bin
sudo chown "$USER:$USER" \
  gxfp-backup/bb010002-before.bin \
  gxfp-backup/bb010003-before.bin
chmod 0600 gxfp-backup/bb010002-before.bin gxfp-backup/bb010003-before.bin
```

Keep those files private. They are useful for analysis and may be needed if you
want to restore the previous sensor state.

## Step 2: try an existing PSK first

If you already have a 32-byte raw PSK for this sensor, install it and test a
single capture:

```bash
sudo install -d -m 0700 /var/lib/open-fprintd/gxfp
sudo install -m 0600 path/to/psk-raw32.bin /var/lib/open-fprintd/gxfp/psk-new-raw32.bin
sudo gxfp_capture_once \
  --psk-raw32 /var/lib/open-fprintd/gxfp/psk-new-raw32.bin \
  --out /tmp/gxfp-test.pgm
```

If capture succeeds, delete the raw test image and skip reprovisioning:

```bash
sudo rm -f /tmp/gxfp-test.pgm
```

Then restart the services and continue with enrollment:

```bash
sudo systemctl start open-fprintd.service gxfp-openfprintd.service
fprintd-enroll -f right-index-finger "$USER"
```

If capture fails with `SSL - Verification of the message MAC failed`, the PSK
does not match the current sensor state.

## Option A: recover the Windows Goodix cache PSK

On systems that previously used Windows fingerprint login, the matching PSK may
be recoverable from the Windows Goodix cache. This is the preferred path when it
works because it does not write new sensor provisioning state.

The helper comes from `Void755/gxfpmoc`. It uses Windows DPAPI, so run it on
the Windows installation that owns the Goodix cache, not from a Linux-mounted
Windows partition.

From a Windows checkout of `gxfpmoc`:

```powershell
git clone https://github.com/Void755/gxfpmoc.git
cd gxfpmoc
py tools\unseal_bb010002.py --out-psk psk-from-windows-raw32.bin
```

The helper auto-searches common `ProgramData` locations. If it does not find
`Goodix_Cache.bin`, pass the cache path explicitly:

```powershell
py tools\unseal_bb010002.py --cache C:\Path\To\Goodix_Cache.bin --out-psk psk-from-windows-raw32.bin
```

Copy the resulting `psk-from-windows-raw32.bin` back to Linux, keep it private,
and test it with the commands from [Step 2](#step-2-try-an-existing-psk-first).

## Option B: invasive reprovisioning

Use this only if no existing PSK works. This builds a new `bb010002` blob,
uploads it to the sensor, and saves the matching raw PSK for the Linux backend.

Build a new provisioning blob and PSK:

```bash
gxfp_psk_tool --build-bb010002 gxfp-reprovision/bb010002-new.bin \
  --out-psk-raw32 gxfp-reprovision/psk-new-raw32.bin
chmod 0600 gxfp-reprovision/bb010002-new.bin gxfp-reprovision/psk-new-raw32.bin
```

Upload the new blob:

```bash
sudo gxfp_psk_tool --upload-bb010002 gxfp-reprovision/bb010002-new.bin
```

Dump the post-upload state for your private records:

```bash
sudo gxfp_psk_tool --dump-bb010002 gxfp-reprovision/bb010002-after-upload.bin
sudo chown "$USER:$USER" gxfp-reprovision/bb010002-after-upload.bin
chmod 0600 gxfp-reprovision/bb010002-after-upload.bin
```

Install the new PSK where the backend expects it:

```bash
sudo install -d -m 0700 /var/lib/open-fprintd/gxfp
sudo install -m 0600 gxfp-reprovision/psk-new-raw32.bin /var/lib/open-fprintd/gxfp/psk-new-raw32.bin
```

Test a single capture before enrolling:

```bash
sudo gxfp_capture_once \
  --psk-raw32 /var/lib/open-fprintd/gxfp/psk-new-raw32.bin \
  --out /tmp/gxfp-test.pgm
sudo rm -f /tmp/gxfp-test.pgm
```

Restart services and enroll:

```bash
sudo systemctl start open-fprintd.service gxfp-openfprintd.service
fprintd-enroll -f right-index-finger "$USER"
```

## Tested PSK location

The backend service expects:

```text
/var/lib/open-fprintd/gxfp/psk-new-raw32.bin
```

The file should be root-only:

```bash
sudo install -d -m 0700 /var/lib/open-fprintd/gxfp
sudo install -m 0600 psk-new-raw32.bin /var/lib/open-fprintd/gxfp/psk-new-raw32.bin
```

## Restore notes

If you need to restore the previously dumped provisioning blob, the mechanical
command is:

```bash
sudo gxfp_psk_tool --upload-bb010002 gxfp-backup/bb010002-before.bin
```

This is not a guaranteed recovery path for every failure mode. Keep the backup
files private and include only redacted logs when asking for help.
