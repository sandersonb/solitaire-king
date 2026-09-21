## ADDED Requirements

### Requirement: Win celebration animation
On a win reached by play, the GUI SHALL play a celebration animation in which the
52 completed cards fall from the four foundation piles and bounce around the
screen. Cards SHALL be released in rank order starting with the Kings and
proceeding down through the ranks to the Aces. Each card SHALL fall straight down
from its foundation position, accelerating under gravity with a small random
variation in acceleration, and on reaching the bottom of the screen SHALL bounce
back up with a randomized horizontal velocity and rotation. A card MAY bounce
several times, losing energy each time, and SHALL eventually come to rest. As each
card launches, the next card still on its foundation SHALL be shown resting in the
slot, so each pile visibly drains from the King down to empty rather than a card
appearing to fall from an empty slot. The overall motion SHALL be visibly chaotic
— scattering cards across the screen — and the whole sequence SHALL run for roughly
6 seconds, so cards fall quickly. When the animation ends (by dismissal or on its
own), the settled cards SHALL remain on screen, beneath the win banner, until a new
game is started (which clears them). The animation SHALL be purely cosmetic: it
does not change game state or scoring.

#### Scenario: Cards cascade from the foundations on a win
- **WHEN** a game is won by play
- **THEN** the foundation cards begin falling — Kings first, then down through the ranks to the Aces — accelerating as they drop, and as each card leaves, the next card still on its foundation is shown resting in the slot (the pile drains from King to empty)

#### Scenario: Cards bounce and settle
- **WHEN** a falling card reaches the bottom of the screen
- **THEN** it bounces back up with a randomized direction and rotation, may bounce again with less energy, and eventually comes to rest

#### Scenario: Celebration is time-bounded
- **WHEN** the celebration has been playing for about 6 seconds without interruption
- **THEN** the animation ends on its own

#### Scenario: Celebration is dismissible
- **WHEN** the player clicks or taps during the celebration
- **THEN** the animation ends immediately

#### Scenario: Settled cards remain until a new game
- **WHEN** the animation has ended and the win banner is shown
- **THEN** the settled cards stay on screen beneath the banner, and are cleared only when the player starts a new game
