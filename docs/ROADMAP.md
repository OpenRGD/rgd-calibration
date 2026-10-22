# OpenRGD Calibration — Roadmap

**Canonical planning checklist:** [Master Development Checklist](CALIBRATION_MASTER_CHECKLIST.md).  
This file is a brief milestone index. The checklist contains acceptance gates, dependencies, negative tests and explicit status.

| Milestone | Focus | Current state |
| --- | --- | --- |
| CAL-0.1 | Rust foundation, immutable profiles, lifecycle, structural evidence | **Merged / CI verified** |
| CAL-0.2 | Passive Core inspection, coverage, read-only proposal, CLI JSON | **Merged / CI verified** |
| CAL-0.2.1 | Console v0.4 local acceptance, parser/canonical hash/registry contract hardening | **Next / not completed** |
| CAL-0.3A | Offline guided session engine and typed metrology contracts | Planned |
| CAL-0.3B | Guided joint/encoder/zero/reference procedures | Planned |
| CAL-0.3C | Guided camera, IMU, force/contact and temporal alignment | Planned |
| CAL-0.3D | Console guided wizard, review, approvals, immutable commit | Planned |
| CAL-0.4 | Runtime-authorized safe active calibration only | Planned; blocked on safety gate |
| CAL-0.5 | Physical drift monitoring and revalidation | Planned |
| CAL-0.6+ | Evidence-driven self-calibration proposals and fleet workflows | Research/future |
| CAL-1.0 | Production QA, provenance, documentation and release governance | Release target |

**Safety boundary:** module installation, RGD import, passive inspection or Console startup MUST NOT trigger motion. The Calibration crate is not a motor controller or safety authority.

**Planning rule:** no physical calibration claim based solely on sample RGD values, hashes, synthetic data or green unit tests. See the [master checklist](CALIBRATION_MASTER_CHECKLIST.md) for verification criteria.
