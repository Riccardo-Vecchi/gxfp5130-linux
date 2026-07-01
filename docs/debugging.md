# Debugging

## Service state

```bash
systemctl status open-fprintd.service gxfp-openfprintd.service --no-pager
fprintd-list "$USER"
```

The stock `fprintd.service` should be masked when `open-fprintd` owns
`net.reactivated.Fprint`.

## Kernel state

```bash
dkms status
modinfo gxfp
journalctl -k -b --no-pager | grep -Ei 'gxfp|GXFP5130|finger|goodix'
```

Expected boot-time lines include `GXFP5130:00` initialization and the firmware
string. Repeated ACPI/GPIO/IPU camera warnings are usually unrelated to this
fingerprint driver.

## Verifying matching

```bash
fprintd-verify -f right-index-finger "$USER"
```

On the tested machine, the final matcher threshold was `5`: the enrolled right
index matched, while another finger did not.

## Raw capture debug

The backend supports an opt-in debug flag:

```ini
Environment=GXFP_DEBUG_CAPTURE_DIR=/root/gxfp-debug-captures
```

Only enable this temporarily in a systemd drop-in. It saves raw PGM fingerprint
captures, which are biometric data. Use a root-only directory (`0700`), copy
only what you need, and delete the files after analysis.

After changing the service:

```bash
systemctl daemon-reload
systemctl restart gxfp-openfprintd.service
```

Analyze a set of captures:

```bash
gxfp-analyze-samples /path/to/pgm-directory
```

## One-shot capture helper

The backend invokes:

```bash
gxfp_capture_once --psk-raw32 /var/lib/open-fprintd/gxfp/psk-new-raw32.bin --out /tmp/finger.pgm
```

Run this manually only in a private directory and delete `finger.pgm` afterward.
