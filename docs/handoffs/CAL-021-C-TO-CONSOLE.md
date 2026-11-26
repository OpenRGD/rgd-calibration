# HANDOFF — CAL-021-C → Console

**Stato:** CAL-021-B integrata in main; CAL-021-C in [PR #9](https://github.com/OpenRGD/rgd-calibration/pull/9), candidato da revisionare. Nessuna attivazione hardware, modifica Core o aggiornamento del Registry. Per evidenza esecutiva consultare i workflow della PR #9 sul relativo head.

Console v0.4.3 @ a0ef857cf425c1bb14ad8f62acec21f8c1324866. Il Module Agent reale viene collaudato con il binario CAL candidato e risoluzione Registry iniettata; non è una prova di installazione o di API/UI live. La versione del report passivo resta 0.2.0; il wrapper resta console.calibration-inspection.v0.4b.

**Priorità per owner Console (PR separata, non deploy):**

- Validare l'intero report CAL con gli ID di schema locali e la dipendenza Measurement, non soltanto le proprietà attuali. La route senza profilo deve richiedere explicit null per profile_id, profile_state, export_preview; profile_binding=no_profile; nessun dato storico inventato. Rifiutare digest malformati e verificare path, role, set e digest con i byte realmente estratti. Conservare gli errori stabili senza esporre stderr arbitrario.
- Il parser JSON5 dell'importer può accettare input non supportati dal CAL-021-A. Riusare il corpus di 22 casi; rendere esplicita l'incompatibilità nella sola funzione Calibration. Non riscrivere lo ZIP o invalidare retroattivamente un modello per altri usi.
- Preparare, con owner Core, un percorso privato e limitato per l'intero spec/ originale (manifest, kernel, tutti i documenti selezionati dal Core), preservando byte e SHA dello ZIP. I cinque file Foundation del vecchio Agent non dimostrano il root canonico. Tenere il nuovo sidecar cal.canonical-identity.v0.1-candidate separato dal wrapper v0.4b. Non sostituire canonical_not_verified con una stringa inventata.
- Conservare accesso loopback/tunnel e isolamento attuale, senza /dev o Docker socket, senza parametri CLI arbitrari, senza --profile/--model-hash/--out sulla route passiva. Nessun intervento su PERIPH, IMU, Voice o publisher.

Richiesta di ritorno: schema e test consumer, parser/archivio, matrice compatibilità e migrazione, esiti API/UI e rollback in D/E. Non aggiornare ancora il pin Registry.
