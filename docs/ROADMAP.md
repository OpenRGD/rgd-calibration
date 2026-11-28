# OpenRGD Calibration — Roadmap

**Canonical planning checklist:** [Master Development Checklist](CALIBRATION_MASTER_CHECKLIST.md).  
This file is a brief milestone index. The checklist contains acceptance gates, dependencies, negative tests and explicit status.

| Milestone | Focus | Current state |
| --- | --- | --- |
| CAL-0.1 | Rust foundation, immutable profiles, lifecycle, structural evidence | **Merged / CI verified** |
| CAL-0.2 | Passive Core inspection, coverage, read-only proposal, CLI JSON | **Merged / CI verified** |
| CAL-0.2.1 | Console/Registry interoperability hardening | **A/B/C/D.1–D.3 merged; D.4 integration candidate; E/F open** — [D.4](CAL_021_D4_INTEGRATION_MIGRATION.md) |
| CAL-RI-01 | RI consumer-reference sidecar (CAL PR #5, RI PR #4 upstream) | **Merged as opt-in Rust feature**; upstream RI Draft; no runtime authority |
| CAL-0.3 OCCHI | Optional experimental stereo/OpenCV tooling (CAL PR #4) | **Merged as isolated experiment**; no physical verification or module release |
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
