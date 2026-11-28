# HANDOFF — CAL-021-D.3 → Registry / Console

CAL-0.2.1 source candidate in CAL PR #13. Consult the exact PR head and successful D.3 CI build evidence; never resolve a moving branch as a release pin. The current distributed CAL-0.2.0 descriptor (81700401e2fa6d370a86b6c1f5a48802bc5f1fc7) remains unchanged. No publication authorized.

- Cargo.toml, Cargo.lock and module.openrgd.json agree on 0.2.1. Passive report remains 0.2.0, profile schema 0.1.0. Default features empty; no dependency/capability/permission changes. `inspect` is the only operational CLI; `--version` gives metadata only (not supported by the legacy registered binary).
- For the next D.4 review, use the accepted immutable source commit, the actual pinned Registry manifest check, and the [CLI contract](../CLI_CONTRACT.md). A CI artifact hash documents that build, not a signature or production install receipt; Registry builds from pinned source and verifies its own installed binary.
- Registry: propose additive descriptor only after package acceptance. Keep old version byte-for-byte and separate installation paths. Console: version-aware consumer contract, verify source/version/binary and keep v0.4b legacy semantics. Never substitute the new executable under an old discriminator. Select explicitly, no automatic fallback; real install/rollback and mixed-version tests belong to D.4/E.
- Preserve historical profiles/cache/source ZIP and no-profile route. Keep optional sidecars/RI experimental code inactive. No access to devices or activation on installation.

D.4 acceptance and release E/F are independent. No external repository edit or deployment requested by this handoff.
