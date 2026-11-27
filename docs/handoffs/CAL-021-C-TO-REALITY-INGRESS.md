# HANDOFF — CAL-021-C → Reality Ingress / RI-H04

**Stato:** CAL-021-B integrata in main; CAL-021-C in [PR #9](https://github.com/OpenRGD/rgd-calibration/pull/9), candidato da revisionare. Nessuna attivazione hardware, modifica Core o aggiornamento del Registry. Per evidenza esecutiva consultare i workflow della PR #9 sul relativo head.

Confrontati RI PR #4 @ 5372f758 (consumer schema frozen), RI-A01 PR #7 e RI-A02 PR #8 @ 161318d2897d07d5628ef502cafba1c7c489fff0. Il nuovo stack conserva i blob originali dello schema 3d92e074 e consumer_ref.py a34f4cbd. CAL esercita davvero EvidenceStore.record, verified_ref_from_store e unverified_ref_from_envelope (camera + generic) su dati sintetici, poi passa le referenze originali al Rust adapter. Il test non esegue resolve_session di A02, né assume una nuova API/authority.

- Preservare RI-H04 snapshot storico (CAL #5, fixture-only). Aggiungere un nuovo record di evidenza al registro RI-A01 dopo aver verificato il head, i blob e il run esatti di CAL #9; tipo pinned_ri_implementation con scope producer-to-CAL synthetic offline, non physical commissioning.
- I limiti maxLength sono ora conteggiati in caratteri Unicode da CAL. Il contratto RI ammette interi non negativi senza massimo, CAL usa u64 e rifiuta overflow: concordare un limite condiviso/versionato, non estendere automaticamente l'authority.
- Validare separatamente observation_ref contro lo schema RI e sidecar contro lo schema CAL. Non reinterpretare local hash o source_index_match come attestazione, session binding fisico o autorizzazione. Nessun aggiornamento di ID, SHA, time, ACL, Chronon o permessi.
- RI-A02 resta resolver locale di correlazione di evidenze fornite da reader autorizzato; non introdurre una seconda risoluzione di identità in Calibration. Perception metric depth resta bloccata, senza nuovo camera opener.

Richiesta di ritorno: compatibilità aggiornata con pin e test review, decisione sui limiti numerici e sul futuro confine A02/reader. Nessun merge automatico.
