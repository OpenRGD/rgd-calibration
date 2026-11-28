# HANDOFF — CAL-REG-021D4 — versioned Registry integration

**CAL #13 merged; D.4 candidate review, publication HOLD.** Exact CAL package source `292d42aab5e04392b736693233811734bffb603e` (accepted by merge `f912a961f55bb051022eadc25e1ebd8b95e8f4ef`). [D.3 CI](https://github.com/OpenRGD/rgd-calibration/actions/runs/46112033149) and [CLI contract](https://github.com/OpenRGD/rgd-calibration/blob/292d42aab5e04392b736693233811734bffb603e/docs/CLI_CONTRACT.md). Current registered 0.2.0 pin stays `81700401e2fa6d370a86b6c1f5a48802bc5f1fc7`. Package 0.2.1, passive report 0.2.0, profile 0.1.0. No new permission or operational CLI verb. Metadata --version new only.

## Request to Registry owner

Coordinate with the existing REG-INDEX-01 #3 and EXT-01B publication/trust/ledger #4–#6, not an alternate registry. Review a new 0.2.1 descriptor as a private candidate using the exact accepted source; do not change the old descriptor or publish a new catalog entry. Confirm whether the current pinned-Git/Cargo distribution can be admitted under the eventual immutable publication ledger (wheel review is not equivalent to this format); report any unresolved trust/provenance requirements explicitly. No bypass of release gates or signed publication claim.

Test real multi-version build/install/resolve and failure/recovery in an isolated prefix; version selection explicit, no overwrite/implicit latest/fallback. Retain old installed receipt and source/binary hashes; source commit and local build digest distinct, not the D.3 CI artifact substituted as a receipt. If using generated index, generate only in disposable review fixture or reviewed candidate branch, not installed catalog. No production install or device access. Ensure source-only/tampered/unknown versions refused. Keep 0.2.0 old pin immutable.

CAL D.4A is independently exercising the actual bootstrap Registry installer in private CI, not changing Registry files. Return exact owner PR/head/test evidence, descriptor review disposition and publication/rollback policy to CAL PR #14. E/F remain explicit gates. No merge or deploy requested.
