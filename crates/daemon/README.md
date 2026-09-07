# TypoMorph daemon

## Interactive input simulation

Run the classifier entirely in user space without `/dev/input`, `/dev/uinput`, D-Bus, or root permissions:

```bash
printf '%s\n' 'the quick brown fox' 'этот быстрый тест' | cargo run -p daemon -- test-input
```

Each input line is retained in a 32-character ring buffer. The command reports the detected language, confidence, whether the simulated layout would switch, and the corrected text buffer.

Raw Linux keycodes can be simulated with a `KEYS` line:

```text
KEYS 16 17 18 18 24
```

The optional `--layout` flag selects the starting simulated layout and `--threshold` controls the minimum confidence required for a simulated switch:

```bash
cargo run -p daemon -- test-input --layout ru --threshold 0.65
```
