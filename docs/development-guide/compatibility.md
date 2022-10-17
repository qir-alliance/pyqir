# PyQIR compatible systems

## Operating systems

PyQIR runs on most x86-64 operating systems that can run Python 3.7+; however,
not all of these systems are equally compatible. Operating systems are grouped
into tiers that represent the level of compatibility users can expect.

- Tier 1 systems are compatible. For tier 1 systems:
  - operating system is used in automated tests
  - installation packages provided for them
- Tier 2 systems should be compatible with PyQIR and can be used relatively
  easily. For tier 2 systems:
  - informal testing may have been done
  - the packages for tier 1 systems will likely work in tier 2 systems

### Tier 1

- Windows Server 2019
- [Ubuntu 20.04](https://wiki.ubuntu.com/FocalFossa/ReleaseNotes)
- [Debian 9](https://www.debian.org/releases/stretch/)
- macOS 11

### Tier 2

- Windows 10
- Windows 11
- [Ubuntu 22.04](https://wiki.ubuntu.com/BionicBeaver/ReleaseNotes)
- [Debian 10](https://www.debian.org/releases/buster/)
- [Debian 11](https://www.debian.org/releases/bullseye/)
- macOS 10.7 - 10.15
- macOS 12
