# CAL-021-C — Contract Validation & Cross-Repository Testing

**Candidate verification wave.** No new production API, module install, canonical contract, physical commissioning or authority. The default CLI still produces passive report v0.2.0. CAL-021-B is merged as an optional local sidecar.

## Pinned verification matrix

| Owner | Exact reviewed source | Executed boundary |
|---|---|---|
| Core | [afe1a831](https://github.com/OpenRGD/openrgd/tree/afe1a831b950a4e6560f0285a0103fc4eaa343b7) | Actual Core algorithm via CAL-021-B on full source; separate Core index and CAL report binding |
| Console | [a0ef857c / v0.4.3](https://github.com/OpenRGD/rgd-console/tree/a0ef857cf425c1bb14ad8f62acec21f8c1324866) | Actual Python Module Agent, ZIP extraction, CLI subprocess and report validation; Registry resolution injected; synthetic API wrapper checked against owner schema |
| Registry | [0268b374](https://github.com/OpenRGD/rgd-modules/tree/0268b3748042d04fd3561553616388368881e04a) | Actual descriptor and registry imports, **not** installer or upgraded distribution |
| Reality Ingress | [161318d2 / RI-A02 candidate stack](https://github.com/OpenRGD/rgd-reality-ingress/tree/161318d2897d07d5628ef502cafba1c7c489fff0) | Actual EvidenceStore, legacy camera/generic envelope reference producers; frozen RI PR #4 consumer schema and implementation blob pins checked, not A02 resolver acceptance |

The fixture exporter `examples/contract_exchange.rs` is test-only and requires the opt-in `ri-candidate` feature. It emits actual Rust serialized sealed profiles, an unknown position, passive reports with/without saved profile and four sidecar types for each accepted reference. Synthetic human-measurement claims exercise the legacy structural lifecycle, NOT real calibration or approved evidence. The normal CAL binary gains no new CLI verbs. The Python exchange suite checks the seven local JSON schemas (five CAL, one RI, one Console), including cross-references using an explicit offline resource registry and format checks.

## Corrections in CAL

- Resolve the passive report's `Measurement` reference using the actual CAL profile schema ID. The previous relative filename did not resolve to the declared profile schema ID by itself. No new report fields or versions.
- Reject duplicate evidence references, as already required by CAL v0.1 schema; unknown zero-reference records with nonempty evidence lists must also refer to existing evidence. Do not automatically mutate/reseal previously accepted profiles that contain duplicates; identify, review and migrate them as new revisions if present.
- RI input length checks use Unicode scalar counts matching JSON Schema min/maxLength rather than UTF-8 byte count. This does not relax any authority flags or digest rules. Rust integer inputs remain bounded to u64, even though RI candidate schema currently has no maximum.
- Schema validation is NOT runtime validation: the legacy reserved `active` enum, integrity seal, references across arrays and lifecycle rules require separate CAL checks.

## Open compatibility gates

- Console v0.4.3 uses a partial report validator. Characterization tests demonstrate accepted missing `export_preview`, unsupported profile context on the no-profile route, and invalid per-file digest. Do not report this as full consumer conformance. Owner must enforce full CAL schema plus no-profile route invariants, and compare actual extracted file digests. Do not change Console automatically from CAL.
- Core-owned hash checks operate on full available spec; no proof of complete/valid schema or signed provenance, nor assignment of a legacy profile's model hash. The five-file Console extraction is insufficient for canonical root verification. Source ZIP hash, canonical root and physical instance remain distinct.
- RI references remain metadata/provenance claims. Upstream schema permits nonnegative integers beyond u64; CAL rejects those. Keep RI-A01 historical evidence and add a newer exact tested record once this PR is reviewed. RI-A02's local session resolver is separate, not called by this test, and must not become a CAL authority or media access path.
- Nested sidecars and wrapper reports require their own owner schema as well as the outer schema. No runtime payload resolver, camera opener, Chronograf anchoring or History commit.
- No version pin changes until CAL-021-D. The released Registry still points to CAL-0.2.0 at 81700401e2fa6d370a86b6c1f5a48802bc5f1fc7. No API server, live Docker/host, UI, signed release or upgrade/rollback test is claimed here.

## Running

See `.github/workflows/cal-021-c-contracts.yml`. Check out the four exact upstream refs into isolated local directories, install the listed test dependencies, build the default binary and the opt-in example, then run `scripts/test_contract_exchange.py` with those paths. Checkout and dependency setup need network; the test uses only local inputs/outputs and no device access.

## Owner handoffs

- [Console](handoffs/CAL-021-C-TO-CONSOLE.md) — consumer hardening and full source snapshot boundary.
- [Core](handoffs/CAL-021-C-TO-CORE.md) — canonical verifier authority and parser/manifest limits.
- [Reality Ingress](handoffs/CAL-021-C-TO-REALITY-INGRESS.md) — real producer cross-test evidence, RI-A01/A02 boundaries.
- [Module Registry](handoffs/CAL-021-C-TO-REGISTRY.md) — defer pin/version migration to D.

Perception and PERIPH do not require code changes in this wave; the existing depth/ownership gates remain in CAL-RI-01. No merge or deployment into an upstream repository is requested.
