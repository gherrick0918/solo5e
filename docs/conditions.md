# Conditions (v1)

Solo5e currently models three common D&D 5e conditions:

- **Poisoned** – attack rolls have disadvantage while affected.
- **Prone** – melee attackers gain advantage; ranged attackers suffer disadvantage.
- **Restrained** – creatures have disadvantage on their attack rolls while restrained, and attackers have advantage against them.

## Duration rules

Conditions can specify duration metadata when applied:

- `duration.until`: optional phase where the effect automatically ends on the target's next turn. Supported values are `"start_of_turn"` and `"end_of_turn"`.
- `duration.save_ends_each_turn`: when `true`, the affected creature attempts the provided saving throw at the end of each of its turns to shake the condition.
- `save`: optional saving throw made immediately on application to resist the condition.

If both `save` and `duration.save_ends_each_turn` are provided, the same saving throw parameters are reused for the recurring saves.

## Example JSON

```json
{
  "kind": "poisoned",
  "save": { "ability": "con", "dc": 13 },
  "duration": {
    "until": "start_of_turn",
    "save_ends_each_turn": true
  }
}
```

This specification applies Poisoned on a failed DC 13 Constitution save, persists until the creature's next turn starts, and offers an additional save at the end of each of its turns.
