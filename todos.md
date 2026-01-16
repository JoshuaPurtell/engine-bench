# Engine-Bench TODOs

Centralized TODOs captured from repo docs.

Progress:
- Created `engine-bench/gold/core_patches/`.
- Implemented Holon's Castform stub logic in `engine-bench/gold/stubs/hp_044_holons_castform.rs`.

- HP Holon mechanics: implement Holon's Pokemon attachment rules, including marker scanning for attached Holon's Pokemon and delta count (active + bench).
- HP Holon mechanics: implement the remaining hooks in `engine-bench/hp.txt` for energy attachment and delta-Pokemon counting.
- HP core infrastructure:
  - Create `gold/core_patches/` directory.
  - Write reference implementation of Holon's Pokemon hooks.
  - Test hooks work with manual implementation.
  - Document hook signatures in `EXTENDING_ENGINE.md`.
- HP tier 1 cards:
  - HP-044 Holon's Castform (full implementation + tests).
  - HP-045, HP-046 if they exist (or skip).
  - Verify core patches work correctly.
- HP tier 2 cards:
  - HP-007 Flygon δ.
  - HP-008 Gyarados δ.
  - HP-010 Kingdra δ.
  - HP-011 Latias δ.
  - HP-012 Latios δ.
  - HP-014 Pidgeot δ.
  - HP-015 Raichu δ.
  - HP-016 Rayquaza δ.
  - HP-017 Vileplume δ.
  - HP-003/004/005 Deoxys δ formes.
- HP tier 3-5 cards:
  - Attack modifier cards (10 cards).
  - Trainer/Energy cards (7 cards).
  - Pre-evolution cards (3 cards).
- HP evaluation:
  - Run full eval suite.
  - Test with multiple models.
  - Document results.
  - Update README.
