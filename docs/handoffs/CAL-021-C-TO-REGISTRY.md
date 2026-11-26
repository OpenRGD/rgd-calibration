# HANDOFF — CAL-021-C → Module Registry

**Stato:** CAL-021-B integrata in main; CAL-021-C in [PR #9](https://github.com/OpenRGD/rgd-calibration/pull/9), candidato da revisionare. Nessuna attivazione hardware, modifica Core o aggiornamento del Registry. Per evidenza esecutiva consultare i workflow della PR #9 sul relativo head.

Il catalogo installabile resta org.openrgd.calibration@0.2.0, source commit 81700401e2fa6d370a86b6c1f5a48802bc5f1fc7. Le wave A/B/C cambiano il codice candidato in main o PR, non la release registrata.

In CAL-021-D servirà una PR dedicata e coordinata con Console: nuovo identificativo versione, pin commit immutabile, manifest Cargo/Registry allineati e dichiarazione interfacce realmente eseguibili. Mantenere passive-report v0.2.0 distinto da versione package; ri-candidate, tool OCCHI, verifier Python opzionale ed example di test non devono diventare comandi installabili o attivarsi in automatico. Rivedere scelta versione esplicita, installazione parallela/rollback, integrità source e binary, compatibilità del Module Agent con vecchia e nuova release. Non sostituire il pin precedente o pubblicare una release prima dei gate D/E/F. Nessuna nuova permission, dipendenza camera o accesso motorio.

Richiesta di ritorno: piano versionato e test di installazione reale in CI e su host supportati; firme/attestazioni restano un gate di produzione separato.
