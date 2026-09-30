# Changelog

Format loosely follows [Keep a Changelog](https://keepachangelog.com/). Entries start here — see [GitHub Releases](https://github.com/artsensiva/TypoMorph/releases) for earlier version notes.

## Unreleased development work

- Add an isolated GTK storage transaction prototype with ordinary undo/redo and
  bounded targeted reversal; retain application/view refusal gates.
- Record layout/view callback compatibility gaps with synthetic refusal tests.
- Replace obsolete landing-page pricing and download claims with development
  status, approved planned pricing and links to the current development branch.
- Refresh README and add durable project recovery instructions.
- Gate automatic replacement while field ownership and input ordering remain unsafe.
- Remove built AI/cloud and obsolete Lemon Squeezy entry points.
- Add signed offline entitlement verification; account/payment service remains pending.
- Add atomic persistent preferences and diagnostic pause reader teardown.
- Bound diagnostic queues and stop on overflow/device loss; redact input-bearing Debug output.
- Reduce browser extensions to content-free readiness with no page access.
- Require signed installer metadata, explicit installation and opt-in autostart;
  omit broad input-device rules from new packages and prepare draft releases only.

The earlier 0.2.2 suppression change dropped genuine events as well as synthetic
ones; it did not establish safe replacement. BUG-001 remains open. These changes
are not a version 1 release or proof of correction in real applications.
