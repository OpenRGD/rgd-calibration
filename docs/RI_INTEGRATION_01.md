# CAL-RI-01 — Reality Ingress ↔ Calibration, candidate association

**Scope:** CAL repo, separate adapter proposal. **Non-normative, read-only, no motor authority, no merge.** Date: 2026-11-25.

## Current source versions and boundaries

- CAL-0.2 is on `OpenRGD/rgd-calibration` `main` at `81700401e2fa6d370a86b6c1f5a48802bc5f1fc7`, with Rust immutable profiles and passive inspection. The CAL-0.1 `CalibrationProfile.state` enum is `uncalibrated` / `calibration_required` / `in_progress` / `validated` / `degraded` / `expired` / `invalid`. `validated` in this software contract is not proof of certified metrological correctness.
- CAL PR [#3](https://github.com/OpenRGD/rgd-calibration/pull/3): master checklist, unmerged.
- CAL PR [#4](https://github.com/OpenRGD/rgd-calibration/pull/4): experimental OCCHI stereo capture/estimation in `tools/occhi_stereo`, unmerged; Python/OpenCV code, no approved CAL-0.1 profile output, no motion. The existing prototype **opens a UVC camera** when explicitly asked to `capture`. Its camera acquisition is NOT a shared device publisher: do not invoke it from RI or when another owner holds the device.
- RI v0.4 is on `OpenRGD/rgd-reality-ingress` `main`; PR [#4](https://github.com/OpenRGD/rgd-reality-ingress/pull/4) is a **draft**, containing the candidate `ri.consumer-observation-ref.v0.1-candidate` and `consumer_ref.py`. This adapter pins the review shape to RI PR #4 head commit `5372f758df79c93fc887f3858d878635447f4a4d`; it DOES NOT pretend that this protocol has been merged or canonically accepted.
- RI's legacy `src/reality_ingress/calibration.py` implements a separate optional intrinsics-only camera experiment. Retain it as legacy compatibility; do not shift its hardware acquisition into CAL or duplicate camera opening.

Ownership: **Host/Peripheral publisher** acquires the frame once; **RI** keeps immutable evidence identity and local-hash/provenance reference; **CAL** owns artifact/revision/quality/validation; **Perception** owns derived stereo depth output with independent quality/unverifiable-frame flags; **Chronograf** owns time qualification; **Somatic** alone can authorize movement under its own safety authority.

## Minimal adapter actually implemented

`src/ri_association.rs` provides `associate_ri_observation(&ri_ref, candidate_artifact_ref, sealed_profile)`.

1. Receives a **copy of the existing RI read-only consumer reference**, never the v0.4 observation envelope/media payload, motor or live camera.
2. Enforces the candidate RI reference shape, flags `source_verified=false`, `robot_binding_verified=false`, `history_committed=false`, `actuation_enabled=false` and **unknown source capture time / null Chronograf anchor**.
3. Associates via an **external sidecar** (`cal.ri-observation-association.v0.1-candidate`). It does NOT change the original `observation_id` or `payload_sha256_hex`, re-key evidence, rewrite a historical observation or persist every frame.
4. References either:
   - `experimental_estimate`, `contract_maturity=experimental` (including future controlled OCCHI experimental artifacts). `calibration_profile_state=null` because there is no approved CAL-0.1 profile.
   - `sealed_profile`, `contract_maturity=candidate`, only when CAL is given a **real sealed CalibrationProfile** whose `profile_id`/`revision` (the profile ID in CAL v0.1), hash, physical instance and model binding match exactly. The state is read from CAL, not inferred from RI.
5. Compares only available RI source/modality/frame/media **claims** with the expected CAL reference; unresolved/changed values produce `incompatible_metadata`. RI v0.1 does not carry verified UVC VID/PID, video resolution, physical A/B orientation or body extrinsics: those remain labelled `claim` or unknown. Local hash integrity is not hardware authenticity.
6. Emits `depth_consumer.metric_depth_allowed=false`, `output_unit=unknown`, unknown A/B physical order, camera→body transform and exposure time. No conversion to meters, no geometry model update, no software/hardware motion permission.
7. No new dependencies, no file writes, no network, no sensor opening, no Chronon creation. Downstream code can persist the sidecar only under **independent ACL, consent and retention policy**; this library never does so automatically.

The JSON contract `contracts/ri-calibration-association.v0.1-candidate.schema.json` describes the additive **CAL** result. It deliberately references the pinned upstream RI schema **in prose**, rather than copying or normatively redefining RI's types: independent RI validation is still required.

## Experimental OCCHI metadata: what is known / what is not

The RI handoff reports **35 stereo image pairs**, **20 mm checkerboard**, **stereo RMS about 0.652 pixels**, baseline estimate **about 64.905 mm** versus a physical lens spacing estimate of **about 60 mm**. This data set is still experimental: A/B physical orientation, stereo metric scale, body/neck extrinsics and actual image trust are not certified.

CAL PR #4's example procedure describes a **25 mm printed board** and accepts an explicit measured square size. These are **not interchangeable sessions**. Every estimate link MUST retain its own board scale/session/procedure, source capture resolution, hardware UID *claim*, relative A/B frame identities and model versus physical instance. If the physical configuration changes (especially a moving neck), the camera-to-neck and neck-to-torso transforms need separate validated observations.

The original stereo UVC SBS format is declared 2560×720, 1280×720 per eye, MJPG transport. A RI v0.4 JPEG may have been **decoded/re-encoded**, so `payload_media_type=image/jpeg` is not proof of original transport bytes, exposure timestamp, dimensions or synchronization. This adapter keeps format as a claim only.

## Perception depth consumer gates — not satisfied by RI or CAL candidate alone

Before enabling camera-frame **metric** depth in a future version, Perception will need independently validated intrinsics and relative stereo extrinsics, camera order and geometry, verified capture resolution/format, baseline/board scale with uncertainty and measured reprojection/validation residuals, exact calibration revision and device identity. For robot-body/world-frame depth it additionally requires validated camera→neck→torso/world transforms and temporal alignment as appropriate.

When one condition is missing/stale/incompatible, Perception must **degrade gracefully** to unavailable/qualitative/unscaled outputs, with explicit units and uncertainty, **not assume meters**. These eligibility rules belong to Perception; CAL only reports evidence and revision state.

## Cross-repo gaps that need an owner decision

- RI: confirm how candidate refs are published to CAL with read ACL/retention and whether `verified_local_hash` is produced by the store or merely reported by a relay. Never grant payload access based solely on this ref.
- Host/Console/PERIPH: confirm the single camera owner and actual UVC frame/format UID binding. CAL PR #4's interactive `capture` is **not** a parallel camera service.
- CAL-0.3: introduce approved typed stereo/transform estimates and revision-specific validation; the experimental Python JSON is NOT automatically a sealed physical profile.
- Perception: define formal `calibration_revision`, transform availability/units, residual/uncertainty threshold policy, quality degradation and stale calibration handling.
- Core/History: independent review of naming/identity/calibration-status and any historical commitment; no candidate contract promotion or invented Chronon from RI/CAL.
- Privacy: keep image datasets and laboratory/face-containing captures **outside Git**, publish only intentionally sanitized report metadata and hashes after consent/retention review.

## Negative tests / rollback

Automated tests must reject malformed RI IDs/digests, unknown schema fields, forged source/robot/history/actuation/time authority, wrong source/frame/media, wrong CAL revision/hash/model/instance, unsealed/tampered profiles, and bogus board sizes. They must preserve the original observation ID, hash and full consumer reference. *No* test opens USB/V4L2, CAN, microphones or moves hardware.

Rollback is simply to **stop producing or consuming the optional association**: RI v0.4 and CAL-0.2 interfaces/records remain unchanged, and no registry/Console release is bumped. Separate PR, no auto-merge.

See [CAL→RI return handoff](handoffs/CAL-TO-REALITY-INGRESS.md).
