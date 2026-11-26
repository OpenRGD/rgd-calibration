# HANDOFF — CAL-021-C → Canonical Core

**Stato:** CAL-021-B integrata in main; CAL-021-C in [PR #9](https://github.com/OpenRGD/rgd-calibration/pull/9), candidato da revisionare. Nessuna attivazione hardware, modifica Core o aggiornamento del Registry. Per evidenza esecutiva consultare i workflow della PR #9 sul relativo head.

CAL usa direttamente il verificatore compute_integrity del Core @ afe1a831b950a4e6560f0285a0103fc4eaa343b7, non un secondo algoritmo. Su un'unica copia privata dello spec confronta anche tutti i digest del report passivo con l'indice Core. Un match del root non implica completezza, schema valido, firma, modello fisico identificato o profilo CAL validato.

Richieste per l'owner:

- Confermare l'API/CLI di verifica read-only supportata e l'identità/versione del toolchain; mantenere distinto OPENRGD_SOURCE_TREE_SHA256_V1 dai digest di file e ZIP.
- Riesaminare i limiti della normalizzazione del self-hash nel manifest e del parser (ambiguità/duplicati e trailing comma): CAL-021-B rifiuta conservativamente i casi ambigui e non corregge sorgenti. Eventuale revisione resta nel Core, con fixture e versione esplicita.
- Definire come combinare indipendentemente verifica di completezza, schema e root, senza promuovere un albero parziale coerente a un modello completo. Nessuna modifica implicita a model_sha256 dei profili CAL storici.
- Tenere separate le questioni già aperte sui riferimenti dei giunti e la policy di calibration_validity; non introdurre alias o modifiche normative nella PR CAL.

Richiesta di ritorno: conferma del confine di autorità o nuova RFC/API candidata con test e pin esatto. Nessuna promozione automatica di cal.canonical-identity.v0.1-candidate.
