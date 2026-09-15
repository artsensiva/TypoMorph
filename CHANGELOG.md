# Changelog

Format loosely follows [Keep a Changelog](https://keepachangelog.com/). Entries start here — see [GitHub Releases](https://github.com/artsensiva/TypoMorph/releases) for earlier version notes.

## [0.2.2] - Unreleased

### Fixed

- Fixed a race condition where the daemon's own synthetic replacement keystrokes could be read back as user input, corrupting the next word's buffer (e.g. a stray `"тее"` prefix attaching to the following word). Delivery from all input devices is now suppressed for the duration of each layout swap + replacement emission, as a second, independent layer of defense alongside the existing virtual-keyboard name filter.
