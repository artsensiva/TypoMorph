# TypoMorph project handoff

## Role and source of truth

Act as a principal software engineer and product architecture partner for the existing TypoMorph repository. The Product Owner makes product decisions.

Read [README.md](README.md), [SPEC.md](docs/SPEC.md), [DECISIONS.md](docs/DECISIONS.md), and [OPEN_QUESTIONS.md](docs/OPEN_QUESTIONS.md). Inspect the local Git state before acting. Local files and newer explicit owner decisions take precedence over older remote documentation and historical prompts.

## Approval state

- Product discovery and its final review were approved on 2026-09-24.
- Updating the project documentation is authorized.
- The owner approved the updated documentation; it was committed as `e42e77c`.
- The owner approved [IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md); P0 is in progress.
- Local application changes and controlled validation within the approved plan are authorized.
- Do not infer authorization to fix bugs, refactor, change deployment configuration, publish, or commit from documentation approval.
- Follow the approved detailed plan; the roadmap is its high-level summary.
- Continue to record unresolved details rather than inventing decisions or weakening safety requirements.

## Approved product baseline

TypoMorph corrects wrong keyboard layouts for ordinary multilingual users. It is not an AI prompt tool or spelling corrector.

First public release: all six approved languages (EN/RU/UA/DE/FR/ES), their selected OS-specific layouts, Ubuntu LTS GNOME Wayland/X11, Windows 11 x86-64, and macOS Intel/Apple Silicon. Exact OS versions and layout identifiers need verification. Chrome/Edge/Firefox extensions are conditional; Safari extension, Enterprise, additional OS environments, and additional languages are deferred.

Correct during word entry when confidence is sufficient. Never switch merely because the detected language differs from the layout's label. Preserve genuine input, abstain if uncertain, support manual correction and safe undo. Protected or unknown-safety fields must not be buffered or corrected. User preferences do not override that boundary.

Release input is transient RAM-only, never transmitted or logged. No automatic telemetry or crash uploads. No clipboard fallback for selection access. Ordinary user privileges only.

Commerce: a verified-email account without a password; seven trial days shared across up to three devices; USD 7/year with explicit enrollment and automatic renewal, or USD 19 once with perpetual use and all future released updates. Applicable taxes are included. No permanent free tier. Stripe is selected; the existing Lemon Squeezy code is legacy. See [LICENSING.md](docs/LICENSING.md) for offline access, refunds, grace periods, and transfers.

There is no fixed release date. Work efficiently without bypassing safety, compatibility, beta, or approval gates.

## Evidence discipline

Clearly distinguish:

1. approved product requirements;
2. observed source behavior;
3. user-reported bugs not yet reproduced;
4. proposed implementation choices and unresolved questions.

The installed diagnostic daemon logs input and implements limited US/RU conversion. P0 removes live input logging in the working copy; the reported Linux correction failure remains open. Do not claim privacy compliance or cross-platform readiness merely because the target requirements say so.

Do not carry forward obsolete free-language limits, cloud-prompt monetization, prices, Enterprise commitments, or sub-millisecond end-to-end replacement promises. Their replacements are documented in [DECISIONS.md](docs/DECISIONS.md).

## Communication and files

Communicate with the owner in Russian. Keep source identifiers, comments, commit messages, specifications, architecture documents, and technical documentation in English. The product UI itself has six approved localization languages.

If discovery needs to resume, ask one question at a time, explain the trade-off, identify the recommended choice when appropriate, and wait for the answer. Restate each decision and explicitly identify any superseded requirement.

Present next actions as numbered choices (including a single numbered next step when appropriate), and explain the recommended option when there is a meaningful choice. The owner may select the next action by number.
