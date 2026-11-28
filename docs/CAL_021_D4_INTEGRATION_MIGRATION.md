# CAL-021-D.4 — Version-aware integration, migration & rollback

**28 November 2026 — engineering candidate, publication HOLD.** D.3 merged in [CAL #13](https://github.com/OpenRGD/rgd-calibration/pull/13), merge f912a961f55bb051022eadc25e1ebd8b95e8f4ef. Use the exact reviewed package head **292d42aab5e04392b736693233811734bffb603e** (reachable from main after the merge), not a moving branch or a future D.4 documentation commit. Its tree was accepted in D.3. [Build evidence](https://github.com/OpenRGD/rgd-calibration/actions/runs/46112033149), CI-only binary SHA-256 fd5b5fd75037fb4bd63221dc4bddaf2c2df8c4220eaa31325f8040530f60eec5. This is not a signature, and installed binaries must be independently verified by the Registry.

## Work split and exact boundaries

| Step | Owner | Result required |
| --- | --- | --- |
| D.4A | Calibration | Real pinned installer/locked build for both CAL versions in private CI catalog/prefix; explicit resolution, rollback, migration characterization; no publication |
| D.4B | Module Registry | Review candidate descriptor against existing generated-index and release-governance work; provide owner-approved path and exact evidence; no actual catalog activation |
| D.4C | Console | Version-aware candidate consumer and rollback UX/API/Agent, actual verified resolution, preserve legacy endpoint; coordinate integrated RC |
| D.4D | Calibration + owners | Record returns/pins, rerun relevant cross-repo tests against version-aware consumer and approved private catalog; decide gate to E |
| E/F | Operators / owners | Supported-host actual API/UI, offline restart, retained volumes and rollback; final release/provenance approval |

The Registry remains at main 0268b3748042d04fd3561553616388368881e04a with org.openrgd.calibration@0.2.0 at 81700401e2fa6d370a86b6c1f5a48802bc5f1fc7. The proposed new source is not enrolled. The Registry owner is independently developing [REG-INDEX-01 #3](https://github.com/OpenRGD/rgd-modules/pull/3), publication review #4, TUF review #5 and [release ledger #6](https://github.com/OpenRGD/rgd-modules/pull/6) at 467a4569901caf85791e637335448ec2f314e7c6. These are draft engineering candidates, not active production trust or distribution. Do not create a parallel publication ledger/catalog or bypass the owner. The bootstrap descriptor supports pinned Git/Cargo; do not infer the newer wheel-oriented publication review automatically covers it.

Console owner [#24 / CONSOLE-025A](https://github.com/OpenRGD/rgd-console/pull/24) at dba4e7d7ed4bcef4f57d8a1510dbaad0f9a96fb9 includes #23 and other owners' work. Preserve the integrated RC; no instruction to merge #23 independently or activate modules in its restricted lab. Version-aware work must coordinate with that owner and its isolated acceptance policy. Do not use production 8088 or a device broker to test this path.

## D.4A real installer verification

The new workflow uses a disposable CI directory, copies the actual pinned Registry runtime and its catalog, appends the 0.2.1 descriptor only inside that copy, then calls the actual Registry CLI to fetch both immutable Git commits and build both passive binaries (Rust 1.90.0, locked, default features). No mocked fetch, build, status or resolution. The original Registry descriptor/index remains unchanged, no version is registered in the repository and the temporary install is deleted. Network is needed for fetch/build. The two private installation receipts and executable hashes identify the real local builds, not signed releases.

- Explicit version required when multiple entries exist; paths remain distinct; duplicate installation refused.
- Verify fresh status/resolve processes with Git/Cargo removed from PATH. This establishes local resolution without the external build tools, not network isolation, a real host restart or browser/API acceptance.
- Run actual resolved passive CLIs against the same synthetic RGD; validate the unchanged report ABI and original source digests; 0.2.1 version metadata is correct, 0.2.0 does not support that flag. Never relabel a 0.2.1 binary as 0.2.0.
- Deliberately tamper the new binary inside the disposable prefix. New resolution and ambiguous selection must fail; explicitly select old 0.2.0 after a fresh digest check. Old installation, profile, model and historical report bytes must remain identical.
- Test historical records created and sealed by the actual old Rust library in a private copy of installed source; compatible profile stays readable, duplicate/dangling evidence cases are refused by new validation. The old CLI may accept records that fail current schema, so do not pass historical output off as new-conformant. No automatic migration or seal repair. The fixture exporter is test-only, not an operational command.

CI emits a small unsigned review evidence JSON with exact source and actual local binary hashes; never export the private descriptor, installed executable or temporary historical records as a release. See workflow and test source for exact assertions. Success on the exact PR head is required; it does not authorize D.4B/C/D or E/F automatically.

## Migration and explicit rollback policy

1. Inventory old release, source pin, verified installed hash, original model archive and saved previews/profiles; preserve the old binary and records. Do not alter seals or rewrite the ZIP.
2. Require explicit approval for a candidate build, exact version/commit selection and verified installed receipt. Keep installation and operational selection separate. No automatic selection of latest or a fallback to 0.2.0 on failure.
3. Version-aware Console must carry selected module id/version through UI → API → restricted Module Agent, compare the actual resolved commit and executable hash, bind output to that selection and return it under a reviewed new wrapper. Validate the actual report schema, no-profile/nulls, source-set/digests and false authority flags. Never accept arbitrary path or CLI arguments. Legacy v0.4b still means 0.2.0 only. Reject incompatible mixed images or absent, source-only, tampered or mismatched installs. For 0.2.1, metadata is a check, not proof of authenticity. Original ZIP hash and Core identity remain different.
4. Keep old cache/profiles read-only. A record rejected by new evidence validation requires reviewed migration, not fabricated data, removed evidence, changed seal or auto-reseal. If an old parent fails current verification, do not blindly invoke the existing revision helper. A later legacy migration adapter needs independent design/review. D.4A characterizes this on synthetic records; inventory of actual operator profiles remains E.
5. On explicit rollback, use the old intact version and its matching legacy consumer, verify receipt/hash before use, preserve both sets of data and do not delete/reinstall automatically. An integrity failure must stay visible until the operator selects a reviewed recovery. E must verify mixed-image and offline restart behavior with retained volumes on supported hosts.

## Handoffs and acceptance

- [Registry — CAL-REG-021D4](handoffs/CAL-021-D4-TO-REGISTRY.md)
- [Console — CAL-CONSOLE-021D4](handoffs/CAL-021-D4-TO-CONSOLE.md)

Request each owner return exact commit, CI and scope, distinguishing code accepted, privately tested, published and deployed. The private installer check is not a replacement for their work. D.4 cannot close until the returns and version-aware consumer regression are reconciled. No version-specific public endpoint/consumer contract is defined by CAL; that is Console ownership. No actual host activation, Registry enrollment, canonical schema promotion, RI feature, PERIPH/Voice/publisher change or physical certification.
