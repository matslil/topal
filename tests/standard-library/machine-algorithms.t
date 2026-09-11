use language (version is v0.1)
use library advent-of-code (version is v0.1)

Pass is Boolean constraint { value } value = true
minimum-indicator-presses is advent-of-code machine minimum-indicator-presses
minimum-counter-presses is advent-of-code machine minimum-counter-presses

manual is machine"[.#.] (0) (1) (2) {1,2,1}
"machine

indicator-minimum : Pass is Pass ((minimum-indicator-presses manual) = 1)
counter-minimum : Pass is Pass ((minimum-counter-presses manual) = 4)

(indicator-minimum, counter-minimum)
