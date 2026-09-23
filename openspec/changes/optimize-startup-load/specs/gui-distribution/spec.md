## ADDED Requirements

### Requirement: Efficient startup asset delivery
The browser build SHALL deliver its card art in a form that minimizes the number of
startup network requests and keeps the transferred size small, so the game reaches
first paint quickly — especially on mobile, where per-request latency dominates.
Card faces SHALL be delivered as a small number of combined images (atlases) rather
than one file per card, and clearly oversized images SHALL be right-sized and
compressed for delivery. Card legibility SHALL be preserved — face art SHALL NOT be
degraded beyond a mild, quality-preserving reduction — and the procedural card
fallback (when art is absent) SHALL remain intact.

#### Scenario: Few requests at startup
- **WHEN** the browser build loads its card art
- **THEN** the faces are fetched as a small number of combined image files, not one request per card

#### Scenario: Oversized art is right-sized
- **WHEN** the build's assets are delivered
- **THEN** images that are far larger than their on-screen size (such as the card back and the logo) are reduced toward their display size and compressed, materially lowering total transfer

#### Scenario: Appearance and fallback preserved
- **WHEN** the game renders cards from the delivered art, or when art is absent
- **THEN** cards look the same as before (faces remain legible) and, if art is missing, the procedural card fallback still renders
