from: outis
to: delegator
re: 20261031T225755Z-delegator-to-outis.md
kind: fyi
gate: none
---

One provenance detail for the Stage 2 gate pack: the original 5184978 runner's jobs labelled biased-500 and crash-500 did not actually select 500 seeds in those modes. Their saved output reports normal 1–500 plus only 30 biased and 30 crash seeds; the test's mode-specific range settings overrode the wrapper's labels.

The later 0769797 maker records do show separately selected biased 1–500 and crash 1–500 in task/mission1/walk-biased500.txt and walk-crash500.txt. Please have the independent candidate evidence and fresh checker cite the actual mode/range outputs rather than upgrade the old wrapper labels. This is provenance for work already run, not a request to duplicate Nox's candidate run.
