# HANDOFF — CAL-021-D.2 → Registry / Console

**Stato:** proposta di compatibilità, non rilascio. La sorgente CAL A/B/C/D.1 è consolidata. Nessun pin aggiornato, nessuna PR esterna modificata.

Riferimento: [matrice D.2](../CAL_021_D2_COMPATIBILITY_VERSIONING.md) e relativo snapshot JSON. **0.2.1** sarà la versione del pacchetto; report passivo **0.2.0** e profilo **0.1.0** restano distinti. La validazione più severa richiede un audit dei profili storici, senza riscriverli. Solo `inspect` è un comando CLI; niente nuovo accesso hardware.

## Registry (D.3/D.4)

- Dopo D.3, ricevere l'esatto commit immutabile con Cargo.toml, Cargo.lock e module.openrgd.json allineati. Non registrare una SHA provvisoria o un branch mobile. Il vecchio descriptor org.openrgd.calibration@0.2.0 / 81700401e2fa6d370a86b6c1f5a48802bc5f1fc7 deve restare intatto.
- Il Registry corrente già gestisce più versioni e richiede la versione esplicita in caso di ambiguità. Confermare policy di pubblicazione/ledger, catalogo additivo e installazione parallela per versione, senza latest implicito, sovrascrittura o fallback. Non creare una seconda source of truth in CAL.
- Limitare l'installazione alla build approvata `--locked --release --bin rgd-calibration` su host supportati. Firme, attestazioni e integrità del runtime/ambiente sono gate distinti. La matrice e i test con descriptor sintetico NON costituiscono una release.
- In D.4/E collaudare entrambe le installazioni reali, checksum e selezione esplicita di rollback; conservare i volumi.

## Console (D.4)

- La PR #23 verifica i report ma Agent/API/wrapper v0.4b sono fissati a module_version=0.2.0. Mantenere il percorso legacy; definire un contratto separato e versionato per selezione esatta di 0.2.1, con identità del pacchetto/commit/binario restituiti dal resolver verificato. Il nome e schema del nuovo wrapper spettano alla Console.
- Non cambiare i campi dei report storici per renderli compatibili. Nuova ispezione esplicita se non idonei. Il parser deve rispettare l'intersezione legacy finché si usa 0.2.0; eventuale profilo 0.2.1 va abilitato e testato per versione. Nessuna conversione JSON5 o modifica dello ZIP. Core identity separata, RI disattivata.
- Rifiutare API/Agent incompatibili e versioni non selezionate/assenti/tampered; no downgrade implicito. Installazione e selezione operativa distinte. Nessun nuovo argv libero o permesso.

Restituire PR e CI esatte per le scelte del consumer/installer, mantenendo la review e accettazione E/F separate. Non è richiesta una pubblicazione immediata.
