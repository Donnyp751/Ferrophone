# Ferrophone

A free, open source softphone that works like a business desk phone: line keys,
BLF lamps, transfer, park and paging, aimed at people who sit behind a PBX
(Asterisk, FreeSWITCH, FreePBX, 3CX). Linux first, then Windows and macOS.

**Status:** planning. Nothing to install yet.

- [Requirements](docs/requirements.md)
- [Architecture](docs/architecture.md)
- [Control API](docs/control-api.md)
- [Decisions](docs/decisions.md)
- [Plan](docs/plan.md)
- [Test environments](docs/testing.md)

Built on [PJSIP](https://www.pjsip.org/) through
[pjsua-rust](https://github.com/Lake-Country-Embedded/pjsua-rust), with a Rust
core daemon (`ferrophoned`) and a Tauri UI.

## License

GPL-2.0-or-later. Release binaries are distributed under GPL-3.0; see
[docs/decisions.md](docs/decisions.md) (D-02).
