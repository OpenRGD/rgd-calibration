# OpenRGD Calibration — Master Development Checklist

**Status reviewed:** 2026-10-22  
**Scope:** `OpenRGD/rgd-calibration` (Rust, offline-first, hardware-agnostic) and its versioned contracts with Canonical Core, Module Registry, Console, Chronograf/Chronon and Somatic Runtime.  
**Rule:** `[x]` means verified in the existing source/CI **for that scope only**; `[ ]` means not yet proven or implemented. Passing synthetic tests never constitutes a physical robot calibration.

## Goal / definition of product

A robot must be able to distinguish its **ideal theoretical body**, **measured physical body**, **uncertainty**, **current position observability**, **reference/rest configurations**, and **calibration lifecycle**. The module proposes or validates evidence-backed *instance corrections*, but does not redefine the ideal RGD model or authorize actuation.

```text
Canonical RGD description (immutable theory)
        + verified physical instance identity
        + independent timestamped physical evidence
                         |
                         v
   Discover -> Inspect -> Prepare -> Measure -> Estimate
                         |
                         v
         Validate -> Commit revision -> Monitor
                         |
                         +--> Calibration Profile + immutable evidence
                         +--> Read-only status/report for Console
                         +--> Proposed instance delta for explicit controlled approval
                         +--> Optional Chronon evidence references (external)
                         |
                   [External Somatic Safety Gate]
                         |
              Motion requests only in CAL-0.4+
```

**Safety invariants for every milestone**

- [x] Calibration has no direct CAN, STM32, Feetech, ROS 2, LeRobot or motor-driver dependency in current CAL-0.2.
- [x] CAL-0.2 `inspect` is read-only; it never moves a robot, commits a correction, asserts a live joint pose or claims physical drift.
- [x] Nominal values, calibration declarations and historically saved measurement claims are separate.
- [x] Synthetic, nominal and derived evidence cannot alone validate a physical profile in the current structural contract.
- [ ] Add permanent negative integration tests proving **load / install / inspect / reboot / malformed input => zero motor commands** across Console and Runtime adapters.
- [ ] Define a fail-closed permission boundary for **any** future motion: valid safety gate, fresh identity/pose evidence, limits, consent, watchdog and safe stop.
- [ ] Prohibit an AI/LLM inference, nominal model, simulator trace or Chronon projection from silently becoming certified physical observation or motor authorization.
- [ ] Reconcile the Canonical Core's calibration-severity conflict via a separate RFC; no silent local reinterpretation.

## Verified baseline — CAL-0.1 Foundation [complete for implementation scope]

- [x] Independent Rust crate, schema-versioned `CalibrationProfile` and stable public library API.
- [x] Instance/model references, hardware/firmware identity fields, observation evidence, scalar measurements with uncertainty and proposed corrections.
- [x] Distinct historical joint zero references and reboot-scoped *ephemeral* position claims.
- [x] Explicit lifecycle states and guarded transitions; active mode rejected.
- [x] SHA-256 profile content seal with immutable, no-clobber local persistence and linked revisions.
- [x] Invalid/missing evidence, changed model/device, stale reboot claims and tampering negative tests.
- [x] Rustfmt, Clippy and 15 foundation tests green when merged by PR #1.
- [x] Canonical Core `spec/01_foundation` audit documenting namespace, policy and field mismatches without modifying Core.

## Verified baseline — CAL-0.2 Passive Inspection [complete for implementation scope]

- [x] Local bounded JSON/JSONC reader for same-version `spec/` or `standard/` Foundation inputs.
- [x] File-byte digests clearly distinguished from canonical model/source-tree identity.
- [x] Joint zero/backlash, sensor extrinsic and camera intrinsic **candidate** field inventory.
- [x] Missing-field coverage, malformed vectors, orphan IDs and unresolved cross-domain references reported without guesses.
- [x] Historical saved measurements tracked independently from RGD declarations; scalar unit mismatch rejected.
- [x] Sealed profile and caller-bound instance/model claims required before optional, unapplied delta preview.
- [x] Candidate `passive-report.v0.2` JSON contract and operator-invoked `rgd-calibration inspect` CLI.
- [x] Pinned Canonical Core BHL example regression: **13 candidate fields, 8 declared, 0 verified physical measurements** for that example only.
- [x] 31 tests, Rustfmt/Clippy, synthetic and pinned-reference CLI smoke green; PR #2 merged.
- [x] Module Registry v0.1 lists `org.openrgd.calibration@0.2.0` pinned to merged source commit, with actual GitHub CI install/build smoke.
- [ ] Console v0.4 deployment on the user's Ubuntu/Fedora hosts: source integration exists, **Docker/UI hardware-host acceptance not yet verified**.

---

## P0 — CAL-0.2.1: Console/Registry readiness and contract hardening

**Why first:** guided/active calibration is premature until the first module can reliably inspect a real **imported** RGD bundle without claiming false identity or measurements.

### CAL-021-A — end-to-end operational acceptance

- [ ] Test Console v0.4 Docker Compose `api + web + module-agent + gateway` on Ubuntu 22.04 mini-PC and Fedora notebook; preserve `console_data` and `module_data`.
- [ ] Verify exact installed Registry pin/version, checksum, module state and immutable source commit through the Console interface.
- [ ] UI flow: browse -> explicit consent -> install -> verify -> select imported RGD -> inspect -> view report.
- [ ] Test offline restart with installed binary, retained imports and no implicit network requirement for inspection.
- [ ] Verify error states: no Git/Cargo/network, missing binary, corrupted install, unverified model, invalid ZIP, disconnected agent, no imported models, unsupported host.
- [ ] Keep the module sidecar without device/Docker-socket mounts and host ports, and keep the Console loopback/tunnel-only until actual authentication exists.

### CAL-021-B — model provenance and parser compatibility

- [ ] Define an exact RGD bundle input contract between Console's JSON5-capable importer and Calibration's JSON/JSONC parser; **reject unsupported syntax explicitly** rather than silently altering input.
- [ ] Implement/consume the Canonical Core's actual `OPENRGD_SOURCE_TREE_SHA256_V1` or accepted equivalent; never substitute an arbitrary raw file digest or a caller assertion.
- [ ] Distinguish `canonical_verified`, `structurally_parsed`, `partial_foundation` and `unknown` in API and UI.
- [ ] Produce a deterministic model-version/physical-instance binding reference; no alias inference from similar joint names.
- [ ] Plan, review and test explicit mapping proposals for `knee_joint_right` vs `leg_right_knee_pitch` and other Core example discrepancies; make **no normative Core change** without approved RFC.
- [ ] Establish structured error codes and a versioned report adapter; unknown fields must never become implicit valid calibration.
- [ ] Add ZIP traversal/size/symlink/decompression-bomb and parser-fuzz negative tests spanning both Console importer and Calibration reader.

### CAL-021-C — contract/release discipline

- [ ] Add Rust <-> JSON Schema conformance fixtures for profiles and passive reports (beyond merely parsing schema documents).
- [ ] Add property/fuzz tests for canonical ref paths, units, NaN/infinite input, truncated evidence, duplicate IDs, corrupted revisions and recursive/symlink paths.
- [ ] Define compatibility matrix `Calibration module version -> Registry descriptor -> Console adapter/report version` and a migration strategy.
- [ ] Keep a single documented CLI JSON contract for Console's `inspect`; library-only Rust capabilities must not be advertised as executable CLI verbs.
- [ ] Define signed/provenance-attested distribution policy for production; commit pin + Cargo.lock + local SHA-256 are **not** independent signatures.

**CAL-0.2.1 acceptance gate:** import an RGD in Console on each supported host, explicitly install the pinned Calibration binary, inspect it read-only, display all unknowns honestly, and survive offline restart. Tests and logs prove **zero actuation**. Do not call this a successful physical calibration.

---

## P1 — CAL-0.3A: Guided Session & typed metrology contracts

**Architecture first, not BHL-specific drivers.**

- [ ] Define `CalibrationSession` with immutable session ID, model and physical instance digests, operator role, started/ended times, environment, mode, procedure/method version and terminal disposition.
- [ ] Define procedural state machine `Discover -> Inspect -> Prepare -> Measure -> Estimate -> Validate -> Commit -> Monitor`, with guarded cancel/abort/failure and idempotent retries.
- [ ] Introduce explicit typed measured quantities: scalar, vector, matrix, transform/quaternion, covariance and strongly specified units/frame IDs; retain backwards compatibility with v0.1 scalar profiles.
- [ ] Specify coordinate conventions: right/left handedness, axes, joint sign, angular wrapping, frame composition order, SI conversion and conversion uncertainty.
- [ ] Define `MeasurementObservation` with sample provenance, raw reading, sampling method, uncertainty, clock domain/boot scope, temporal uncertainty and quality flags.
- [ ] Define `CalibrationEstimate` separate from `CorrectionCandidate`, with algorithm version, input evidence refs, residuals and confidence/uncertainty; do not confuse confidence with a certificate.
- [ ] Define `ValidationProtocol` and `ValidationResult` with tolerance source, independence from estimation, hold-out evidence, repeatability, outlier policy and explicit pass/fail reasons.
- [ ] Define `CalibrationCommit` as a new immutable instance revision, linked to prior profile and approved evidence; default deny on conflicting model/hardware/firmware identity.
- [ ] Specify evidence authenticity and integrity options (signed/attested external receipt where available); distinguish an unverified human/sensor claim from measured proof.
- [ ] Add persistent session recovery/cancellation/crash consistency, append-only audit trail and idempotent commit without overwriting previous revisions.
- [ ] Support physical, guided/manual, simulated and replayed *inputs* but never cross-promote evidence origins; tests must fail if simulation alone is presented as verified physical truth.
- [ ] Add versioned schema and backward-compatibility tests on v0.1/v0.2 stored records.

**Gate:** a fully synthetic **guided session** can start, collect labelled inputs, estimate, independently validate, reject bad input and save a revision offline — **without sending any motion command**.

## P1 — CAL-0.3B: Guided joint, encoder and reference calibration

- [ ] Represent explicitly and individually: encoder zero, mechanical zero, kinematic/model zero, **currently observed pose**, initial reference pose, safe rest pose and homing procedure.
- [ ] Specify encoder type (absolute/incremental/multi-turn), resolution, turn-count observability, offset, direction and power-cycle/boot validity.
- [ ] Guided joint-axis polarity and transmission ratio/sign identification from **independently measured or manually positioned** poses, not inferred from commanded motors.
- [ ] Detect ambiguous/missing absolute reference after restart; produce `POSITION_UNKNOWN` rather than reusing previous pose or assuming mechanical zero.
- [ ] Validate repeatability and uncertainty of zero offsets with independent samples, measured joint/frame conventions and configurable device-specific acceptance tolerances.
- [ ] Record measured mechanical limits and soft safety envelopes **without automatic contact with end stops**; runtime safety limits stay independently authoritative.
- [ ] Keep backlash/hysteresis, friction and compliance hypotheses separate from verified measurements; report what method supports each estimate.
- [ ] Support multi-DOF, coupled and multi-actuator joints through versioned abstract relationships, without assuming a 1:1 motor/joint identity.
- [ ] Add discrepancy reports for actuator/joint identifiers and topology mappings, with explicit human-approved alias proposal workflows.
- [ ] Produce per-joint coverage: unknown / insufficient evidence / estimated / independently validated / degraded; no global invented readiness score.
- [ ] Run first **passive physical-data** acceptance with SO-101 and Paky-01 isolated components as adapter conformance tests, not as hard-coded library dependencies.

**Gate:** after a simulated reboot and missing absolute sensor, status remains unknown; after independent measured evidence, an offset candidate can be reviewed and committed as an instance correction, never as an instruction to move to zero.

## P1 — CAL-0.3C: Guided sensors and spatial/temporal alignment

- [ ] Camera intrinsics: target/board metadata, lens/distortion model, focal/center values, reproducible observation sets, reprojection residuals and uncertainty.
- [ ] Camera extrinsics: sensor-to-body and external-camera-to-world frames, rigid transforms and evidence-based estimation; do not confuse external environment view with robot POV.
- [ ] IMU orientation, gravity alignment, accelerometer/gyroscope bias, scale/cross-axis errors and thermal/context dependence when observable.
- [ ] Force/torque sensor zero/tare and sign; optional pressure, tactile and foot contact calibration with source-specific evidence and uncertainty.
- [ ] Sensor-to-sensor and command-to-feedback latency candidates with clock-domain ID, offset uncertainty, reboot scope and sampling provenance.
- [ ] Use **Robot Chronograf/Chronograph** as temporal authority only if its actual stable APIs are available; local receipt time must remain marked as unsynchronized otherwise.
- [ ] Validate frame graph consistency, impossible transforms, disconnected frames and unresolved sensor IDs without writing hardware configuration.
- [ ] Add typed vector/pose/matrix canonical delta preview once its semantics, acceptance and compatibility tests are approved.
- [ ] Exercise passive/guided fixtures on Logitech C920 POV + external observation and at least one IMU **only when physical evidence is actually collected**; never recycle nominal camera specs as measurements.
- [ ] Preserve temperature, payload, mounting and device replacement as validity conditions; trigger revalidation after changes.

**Gate:** a sensor calibration report identifies exact frames, capture procedure, uncertainties and residuals, supports independent verification, and makes no claim of synchronization absent an authoritative time source.

## P1 — CAL-0.3D: Console wizard, review and commit

- [ ] Add Console UI for calibration session creation, method selection, preflight checklist, acquisition status, measurement upload, review and explicit approve/reject.
- [ ] Show nominal vs observed vs estimated vs committed values with units, uncertainty, provenance and linked evidence.
- [ ] Expose per-component blockers and unavailable procedures instead of offering fake buttons or silently filling defaults.
- [ ] Provide a clear immutable before/after diff, operator attribution and rollback **via a new revision**, never by rewriting history.
- [ ] Define a controlled export/import contract for **instance-only** RGD calibration overlays; do not edit canonical `description.jsonc` or normative schemas in this module.
- [ ] Keep revisions local-first and recoverable offline; optional Chronon links are external pointers, not synthetic ledger entries.
- [ ] Publish/version the module via `rgd-modules`; update the Console adapter only after schema/CLI compatibility verification.

**Gate:** a human can complete a no-motion guided calibration from Console with audit-quality evidence and reject/rollback paths.

---

## P2 — CAL-0.4: Safe Active Calibration (OFF by default)

**Hard stop:** do not start any physical actuation work until an external Somatic Runtime/Safety Gate can prove the required capabilities. A calibration module must NEVER write CAN, motor commands or bypass a safety controller.

- [ ] Finalize a versioned `CalibrationMotionRequest` / `AuthorizedMotionResult` contract with an external Somatic Runtime authority; no raw motor IDs, bus frames or device-specific logic.
- [ ] Require explicit operator consent, configured test mode, hardware identity, current boot/clock, **fresh absolute pose evidence** or independently approved homing method, and safe test fixture.
- [ ] Gate requests against body/collision model, validated per-joint position/velocity/torque/acceleration/jerk limits, payload and environment.
- [ ] Establish bounded micro-motion amplitude/rate/time budgets, monitoring of expected effect and observed residuals, and independent emergency-stop/communication-loss handling.
- [ ] Define deny-by-default watchdog, lease/expiry, timeout, replay protection, human-presence policy and software restart behavior.
- [ ] Ensure the 1 kHz Somatic control loop does not block on calibration storage, UI, network or cognitive inference.
- [ ] Test safety-denial paths: unknown pose, expired calibration, mismatched model or controller, lost communications, E-stop, out-of-envelope request, poor observations and time uncertainty.
- [ ] Exercise SIL/simulation fault injection first; then hardware-in-the-loop and a physically supported, isolated low-risk joint with a human-approved procedure.
- [ ] Allow a single physically constrained experiment only after a separately verified safety checklist and explicit permission; a safe-rest pose is **not** equivalent to zero and returning to it is not a substitute for E-stop.
- [ ] Record the full authorization chain, commanded abstract capability, actual observations, procedure limits, cancellation and rollback evidence.
- [ ] Require independent post-movement validation before a proposed correction is approved; never apply parameters from the same noisy sample uncritically.
- [ ] Keep any machine-specific driver under a separate Body Adapter/Synapse repository, never directly in `rgd-calibration`.

**Gate:** repeated negative tests demonstrate that every missing authority or unknown physical condition **prevents movement**; test-bench acceptance is explicitly signed off separately from software CI.

---

## P3 — CAL-0.5: Continuous Calibration & drift

- [ ] Define per-component validity policies tied to real measurement methods, uncertainty, operating conditions and declared tolerance sources, not arbitrary universal thresholds.
- [ ] Detect revalidation triggers: motor/encoder replacement, mechanical changes, firmware update, model change, reboot without absolute reference, impact, thermal shift and sustained residual deviations.
- [ ] Compare **compatible timestamped physical measurements** across revisions; distinguish sensor noise, changing load, calibration drift and model changes.
- [ ] Define statuses uncalibrated, required, in progress, validated, degraded, expired, invalid, and enforce controlled transitions without implicit success.
- [ ] Add passive observation ingestion and explainable drift report (metrics, intervals, uncertainty, source IDs, decisions and confidence limitations).
- [ ] Provide a degraded/expired result to Console and external Runtime policy consumers; do not itself decide motor authorization.
- [ ] Keep a human-approved revalidation schedule and notification policy; never auto-commit unknown or unverified corrections.
- [ ] Add longitudinal replay over physical and synthetic streams with clearly labelled origins, missing data and repeatability metrics.
- [ ] Link events to authoritative Chronons/HyperAion only when production contracts exist; corrected projections must not rewrite original evidence.
- [ ] Validate reboot and partial-hardware-replacement cases, including preservation of physical-instance history without accidentally reusing obsolete device calibration.

**Gate:** replayed labelled drift scenarios cause correct deterioration/revalidation status without changing motor parameters, overwriting history or inventing measurements.

## P4 — CAL-0.6+: Evidence-driven self-calibration and fleet use

- [ ] Introduce bounded calibration-plan suggestions based on known observability and missing information, with reasons and required sensors.
- [ ] Prioritize experiments by expected information gain, physical risk, permitted authority and time budget, not only by an AI confidence score.
- [ ] Treat HyperAion learning, simulated/dreamed traces and prediction as **hypotheses** until independently observed and validated.
- [ ] Permit autonomous experiment proposal only through the same external safety gate as CAL-0.4; explicit policy decides human approval requirements.
- [ ] Reuse **procedures and generic models** across SO-101, Paky-01 and future robots, but never transplant instance-specific offsets without independent measurement.
- [ ] Define fleet/multi-robot profile lookup scoped to authenticated physical instance + model + device/firmware identity, with conflict detection.
- [ ] Support versioned compatibility/migration and provenance-preserving revocation of bad calibration estimates.
- [ ] Benchmark convergence, uncertainty reduction and reproducibility on independently collected physical datasets.

**Gate:** any autonomous suggestion can be refused without side effects; no autonomous motor/write privilege is derived from cognitive or memory outputs.

## P5 — CAL-1.0: Production and evidence readiness

- [ ] Complete documented schema governance, backward-compatible migrations, semver, changelog and long-term support policy.
- [ ] Separate release maturity (`candidate` vs `accepted`) from whether a given physical instance has a trustworthy calibration.
- [ ] Define reviewer-approved acceptance tests and measured tolerances for every implemented calibration method, with units, uncertainty budgets and independent test sets.
- [ ] Harden file/ZIP/JSONC parser attack surface, local permissions, audit retention, path/symlink attacks, recovery and install-time supply-chain risks.
- [ ] Reproducible CI matrix: formatting, lint, units, schema conformance, property/fuzz, integration, Console end-to-end, registry pinned install and offline restart.
- [ ] Signed release artifacts and provenance, protected branch/review process, rollback protocol, and incident/vulnerability handling.
- [ ] Host-level tests on Ubuntu 22.04 and Fedora for documented supported architectures; driver support remains external.
- [ ] Publish a device/procedure compatibility matrix with explicit unsupported/unverified cases, not a blanket “all robots supported” assertion.
- [ ] Run traceable physical trials covering at least two different robot embodiments and multiple fresh boots, with manual safety review for any moving tests.
- [ ] Produce complete developer/user/operator guides: quickstart, offline backup, diagnostics, migration, measurement protocol, limitations and dispute/recalibration workflow.
- [ ] Define privacy and retention for operator identities, images, physical measurements and immutable event references.
- [ ] Consider formal metrological/safety claims **only** after an independent standards, evidence and hardware safety assessment; software unit tests never constitute safety certification.

---

## Current key blockers / cross-repository ownership

| Blocker | Owner of the solution | Required before |
| --- | --- | --- |
| Console v0.4 Docker/manual acceptance | OpenRGD Console + local operator | CAL-0.2.1 accepted |
| JSON5/JSONC importer compatibility and canonical bundle hash | Console importer + Canonical Core tooling | Trusted model/instance binding |
| Joint-ID aliases and contradictory calibration-validity policy | Canonical Core RFC + reviewed resolution | Automated Core mapping or motion policy |
| Structured vector/pose measurements, session/evidence contracts | `rgd-calibration` | CAL-0.3 sensor procedures |
| Hardware UID, firmware and boot-pose truth | External Body Adapters/Synapse; verified physical evidence | Physically validated joint reference |
| True clock-domain synchronization | Robot Chronograf/Chronograph | Certified temporal correlations |
| Safety-gated motion execution with stop/watchdog | Somatic Runtime + physical safety system | **Any** CAL-0.4 motor test |
| Append-only canonical time/experience history | Chronon/HyperAion producer, when real | Canonical memory linkage |
| Versioned distribution and Console CLI compatibility | `rgd-modules` + Console | Each released module wave |

### Gate definitions

| Gate | What must be demonstrated |
| --- | --- |
| G0 — Source | Correct Canonical Core model and source identity; no mixed/guessed mappings |
| G1 — Instance | Independently supported instance/hardware/firmware bindings and boot-scope, not only typed caller claims |
| G2 — Measurement | Raw evidence, method/version, units, frames, timestamps, uncertainty and provenance |
| G3 — Validation | Independent hold-out / tolerance acceptance and immutable reviewed commit |
| G4 — Movement | External runtime safety permit + fresh pose + E-stop/watchdog + bounded, authorized operation |
| G5 — Monitor | Repeated comparable observations, drift diagnosis, controlled degradation and recalibration |

**CAL-0.1/0.2 implemented only subsets of G0-G3 as structurally claimed software contracts. They do not satisfy G1/G3 as physical attestation, G4, or G5.**

## Test/evidence matrix

| Input | Expected result | Never infer |
| --- | --- | --- |
| Pinned canonical BHL reference sample | Read-only candidate inventory + mismatch findings | Real Paky-01 measurements |
| Synthetic RGD with known correct values | Deterministic expected estimates and validation | Real physical correctness |
| Missing/ambiguous joint name | Explicit unresolved/missing state | Fuzzy alias mapping |
| Incremental encoder after reboot | Position unknown until a valid current reference | Last saved value as present pose |
| Wrong device/firmware/model | Invalid/degraded binding, no commit or motion | Previous calibration still applies |
| Noisy/inconsistent measurements | Uncertainty/outlier flag, rejection when outside protocol | Pass based on one convenient reading |
| Simulated “physical sensor” field without trusted attestation | Provenance claim only; review required | Certification from a string |
| Loss of communication / E-stop in future active test | External safety gate denies/halts operation | Automatic recovery by returning to zero |
| Changed temperature/payload/mounting | Policy-defined revalidation warning | Forever-valid offsets |
| Console offline restart | Read-only report works if installed data valid | Any automatic download/motor initialization |

## Priority: next three implementation PRs

1. **CAL-0.2.1 / Contract & Console interoperability:** pin the v0.4 Console acceptance fixture, finish canonical importer/hash checks, support clean report/unknown-state UX and negative end-to-end tests. *No motor controls.*  
   **Definition of done:** real Ubuntu/Fedora Compose smoke evidence, false-claim tests, and compatible Registry/Console JSON version.
2. **CAL-0.3A / Sessions & typed measurements:** independent offline session lifecycle, typed SI quantities/frames/covariance, evidence provenance, separate estimator/validation/commit contracts, and immutable migrations. *Synthetic only.*  
   **Definition of done:** full guided lifecycle synthetic tests, rejection/tampering/rollback/concurrency checks and no physical actuator access.
3. **CAL-0.3B / Joint reference guided proof:** generic joint zero/encoder/reference workflow and reboot uncertainty cases, with non-actuating physical observation only where actually available.  
   **Definition of done:** unknown-after-reboot remains unknown; approved measured offsets are traceable, repeatable and instance-specific.

Each implementation PR updates this checklist, documents evidence/test outcomes, includes explicit negative/safety tests, and updates Registry/Console compatibility **only** when its interface changes. Do not merge motor-related functionality based only on green software CI.
