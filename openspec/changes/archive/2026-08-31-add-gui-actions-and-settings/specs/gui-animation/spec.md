## MODIFIED Requirements

### Requirement: Drivable by automated playback
The animation subsystem SHALL be usable to play a queued sequence of moves with the
same motion, independent of pointer input, so an automated-play feature (such as
auto-solving a deal) can enqueue moves and have them animate in order. Queued
playback SHALL advance at a fixed cadence — roughly half a second between moves —
rather than applying moves instantly, so the sequence is watchable.

#### Scenario: Queued moves animate in order
- **WHEN** a sequence of moves is enqueued for automated playback
- **THEN** each move is applied and its card motion animates in the given order

#### Scenario: Playback is paced
- **WHEN** a queued sequence is playing back
- **THEN** successive moves start about half a second apart rather than all at once
