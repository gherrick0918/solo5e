# Life, Death, and Recovery

Creatures track hit points with a `Health` record that captures their current HP, maximum HP, and life state.

## States

* **Conscious** – the default state; the creature can act normally.
* **Unconscious { stable }** – the creature is at 0 HP. If `stable` is `false`, they must make death saving throws at the start of each turn. If `stable` is `true`, they are no longer rolling saves but still unconscious at 0 HP.
* **Dead** – no further rolls; the encounter is over for the creature.

## Dropping to 0 HP

Damage is applied with `apply_damage`, which clamps HP at 0 and moves the creature to the unconscious state. The first time this happens the Prone condition is also applied for flavor.

## Death Saving Throws

At the start of an unconscious creature’s turn (while not stable), roll a d20:

* **20** – the creature springs back to life with 1 HP and becomes conscious.
* **1** – two failed death saves are recorded.
* **10–19** – one success.
* **2–9** – one failure.

Reaching three successes stabilizes the creature at 0 HP (no more rolls). Reaching three failures kills the creature.

## Healing

`heal` restores HP up to the maximum. If the creature was unconscious at 0 HP and receives healing, it regains consciousness and its death save counters reset.

## Convenience Flags

* `--auto-potion` automatically heals the actor for 7 HP the first time they drop to 0 HP during a duel or encounter.
* `--short-rest` grants the actor 5 HP of healing (via `heal`) after the duel or encounter ends.
