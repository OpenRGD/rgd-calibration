# HANDOFF DI RITORNO — CAL → Reality Ingress / RI-INTEGRATION-01

**25 novembre 2026 · candidato · non normativo · non attuante.** Adapter CAL in `main` solo se accettato, con feature Rust `ri-candidate` **disattiva per default** e senza attivazione in Console/Registry. PR CAL #3/#4 già integrate, #4 soltanto come esperimento.  
Riferimenti: [RI PR #4](https://github.com/OpenRGD/rgd-reality-ingress/pull/4), [CAL PR #3](https://github.com/OpenRGD/rgd-calibration/pull/3), [CAL PR #4 OCCHI](https://github.com/OpenRGD/rgd-calibration/pull/4).

## Decisioni CAL

- CAL-0.2 su `main` e CAL-0.3 OCCHI **sperimentale e isolato** ora su `main` restano contratti distinti; la PR OCCHI #4 è stata integrata esclusivamente come tool opzionale. Niente promozione a `validated`/`accepted` solo perché esistono uno studio OpenCV o un hash.
- Nuovo adapter **read-only** in repo CAL: `associate_ri_observation(ri_ref, optional_cal_artifact_ref, optional_sealed_cal_profile)`; produce un **sidecar candidato** `cal.ri-observation-association.v0.1-candidate`, non modifica RI v0.4.
- L'`observation_id` e il `payload_sha256_hex` originali rimangono identici. `verified_local_hash` è soltanto integrità verificata dal **RI store**, non una prova di origine/sensore/metrologia. Ogni altro potere resta falso.
- Profilo CAL identificato separatamente da `artifact_id/revision/artifact_sha256`; per un CAL-0.1 sealed profile, `revision == profile_id` e si verifica il sigillo; per OCCHI sperimentale, `kind=experimental_estimate`, maturità `experimental` e **nessuno stato CAL validato**.
- Hardware UID, SBS format/resolution, A/B frame, 20 mm board/session, modello e istanza fisica sono metadati/claim distinti, mai prove implicite. Nessun secondo camera opener.
- Depth consumer **bloccato** in v0.1: unità `unknown`, A/B fisico `unknown`, camera→corpo `unknown`, tempo sorgente `unknown`, `metric_depth_allowed=false`. Perception deciderà i futuri gate e la degradazione.

## Compatibilità / test

- Vecchio envelope e hash RI v0.4 invariati; nessuna scrittura RI/Chronon/Core.
- Test sintetici positivi e negativi per revisione assente/incompatibile, fonte/frame/media, digest, authority flag, timing, profilo CAL contraffatto, deriva/stato scaduto, no side effects.
- La PR è indipendente dalle PR CAL #3/#4 e dalla PR RI #4; nessun merge automatico.

## Punti ancora aperti per RI/Perception e owner

1. Confermare il trasporto/read ACL del consumer ref e il comportamento a retention/expiry (reference ≠ payload access).
2. Confermare chi apre una sola volta OCCHI e consegna frame/refs al fan-out, senza seconda acquisizione nel modulo CAL.
3. Contratto Perception per misure depth derivate: `calibration_revision` + qualità, unità, trasformazioni disponibili e fallback non-metrico.
4. Metrologia CAL-0.3 stereo, baseline e checkerboard reale: dataset RI handoff 20 mm ≠ procedura d'esempio 25 mm in CAL PR #4; identità/scala della sessione da verificare prima di riusare stime.
5. Authority temporale/History resta esterna (Chronograf/Chronon), nessun finto anchor.

**Richiesta:** annotare questa risposta nella matrice RI PR #4 come compatibilità *candidate-only*, non green light per camera reale, streaming continuo, depth in metri, deploy o attuazione.
