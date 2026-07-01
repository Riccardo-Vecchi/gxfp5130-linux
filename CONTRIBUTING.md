# Contributing

Thanks for helping map GXFP5130 Linux compatibility. The most valuable
contributions right now are hardware reports, install fixes, packaging fixes,
and careful debug data.

## Compatibility Reports

Open an issue or PR with:

- laptop model
- BIOS/firmware version if available
- sensor ACPI ID
- firmware string reported by `gxfp`, if available
- distribution and version
- kernel version from `uname -a`
- whether `gxfp` loads
- whether `gxfp_capture_once` can capture
- whether `fprintd-enroll` works
- whether `fprintd-verify` accepts the enrolled finger and rejects another one

Useful commands:

```bash
uname -a
journalctl -k -b --no-pager | grep -Ei 'gxfp|GXFP5130|finger|goodix'
fprintd-list "$USER"
systemctl status open-fprintd.service gxfp-openfprintd.service --no-pager
```

If your hardware works, add a row to the `Tested Hardware` table in
`README.md`.

## Bug Reports

Include:

- the exact command that failed
- the full error text
- relevant service logs:

```bash
journalctl -u gxfp-openfprintd.service -u open-fprintd.service -n 120 --no-pager
```

- relevant kernel logs:

```bash
journalctl -k -b --no-pager | grep -Ei 'gxfp|GXFP5130|finger|goodix'
```

## Safety Rules

Never attach or paste:

- PSKs
- `bb010002` / `bb010003` blobs
- factory dumps
- provisioning traces
- raw PGM fingerprint captures
- enrolled feature/template files

If you enable `GXFP_DEBUG_CAPTURE_DIR`, keep it root-only, use it briefly, and
delete captures after analysis.

## Pull Requests

Keep PRs focused. Good PRs usually do one of these:

- add a tested hardware row
- improve install docs for a distribution
- fix backend behavior with logs and test output
- improve scripts without changing provisioning defaults
- add safer debug tooling

For code changes, mention how you tested them.
