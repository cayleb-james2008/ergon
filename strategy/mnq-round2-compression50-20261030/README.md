# Round-two MNQ compression 50% — stored record export

`survivor-59-trades.csv` presents all 59 saved test-fold trades and their independently recorded baseline and doubled-cost results. The scenario fills can differ: both prices and exit fields are kept separately. `survivor-original-events.json` contains the original event objects and stored scenario summaries for exact traceability. `fold-boundaries.json` is copied from the saved result, and `stored-compression-variant-summary.csv` contains the original stored summary rows for both compression thresholds in MNQ and MES.

The frozen manifest, evaluator, candidate definitions, risk replay and original checker ledger are included as stored. The protocol is a redacted publication copy with local paths removed; its original source hash remains in provenance. `provenance-and-missing-fields.json` provides neutral source labels, original SHA-256 values, field limits and matching rules. The formatted times represent saved ET minute values. In particular, a stop's exit minute identifies a one-minute bar; the actual time within that bar is not recorded. No prices, fills or P&L were reconstructed and no backtest was rerun. These are exposed historical model records, not actual fills or evidence of live profitability.

In the saved side field, `1` means long and `-1` means short.
