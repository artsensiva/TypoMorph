# Licensing, accounts, and commerce

Status: product rules and documentation approved, 2026-09-24.
This is a product specification, not a published customer contract or tax determination. The existing [source license](../LICENSE) remains unchanged.

## 1. Offers and feature access

| Offer | Price | Duration | Computers | Features and updates |
| --- | --- | --- | --- | --- |
| Trial | No payment card | Seven days from the account's first activation | Three | Full features; updates during valid trial |
| Annual | USD 7, applicable taxes included | One paid year; automatic renewal after explicit enrollment | Three | Full features and updates during valid access |
| Perpetual | USD 19 once, applicable taxes included | No expiry | Three | Full features and all future released updates |
| Permanent free tier | Not offered | N/A | N/A | No continued correction after trial expiry |
| Enterprise | Deferred | N/A | N/A | No managed-fleet or separate corporate offer |

There are no paid language packs and no difference in safety protections between trial and paid access. The license covers one person's personal and commercial work use; sharing with colleagues is not allowed. Product names such as "Pro" are not required by the approved model.

Perpetual purchase is not restricted to one major version or twelve months of updates. It does not promise perpetual development, compatibility with every future OS, or personal support forever. Annual and perpetual users receive the same update stream.

## 2. Account and trial

- Verify an email address using a code or sign-in link, without a TypoMorph password.
- Start the account's single seven-day trial on first successful online activation.
- Associate additional installations with that account; all share the original trial end.
- A trial can use up to three computers. Adding a computer never restarts the clock.
- A card is not required during trial; trial expiry alone cannot cause a charge.
- Each installation has a random identifier. No hardware fingerprint is collected.
- Accept that this does not fully prevent new-account or local-state abuse. Do not add hidden device fingerprinting to close that gap.

The earlier proposal for an accountless per-installation trial is superseded.

## 3. Offline operation

| State after successful activation | Required offline behavior |
| --- | --- |
| Trial | Work until the account-wide trial end |
| Annual, paid | Work until the confirmed paid-through date, without periodic online checks |
| Perpetual | Work indefinitely, without periodic entitlement confirmation |
| No valid access | Disable all correction functions; leave ordinary typing unaffected |

Network access is needed to activate a new installation and to obtain downloads. A renewed annual period must be delivered to the installation before its old confirmation expires; an offline application cannot assume that a future payment succeeded.

The local verifier contract below is implemented under the owner's authorization to continue version 1 development autonomously. Production key management, clock-change handling, recovery, and account-service availability remain incomplete. A local unsigned checksum is not evidence of server-issued access.

## 4. Renewals, cancellation, and failure

After an explicit purchase, the annual plan renews automatically at the agreed USD price. Clearly show the recurring amount/date, provide advance notice, and make cancellation accessible. Exact reminder timing is open.

Cancellation stops future renewal, not already paid access. Cancellation does not disable ordinary keyboard input.

A confirmed failed automatic renewal grants seven additional days. Notify the user and offer payment-method repair. The exception does not apply to trial expiry or voluntary cancellation. After that grace expires without payment, disable all correction.

The service-to-offline-client mechanism for confirming and delivering grace must be designed; do not assume an unreachable client can distinguish cancellation from a failed charge.

## 5. Perpetual conversion

A subscriber purchasing perpetual access pays the full USD 19. No annual-payment credit, prorated discount, or refund is implied by conversion. On successful conversion, stop future subscription renewal and preserve access. The operation must have a reviewable payment/entitlement outcome; transactional details are to be designed.

## 6. Device transfer

Users can release an old activation and activate another computer themselves. Activation slots are account-wide and capped at three; the mechanism must handle lost or replaced computers without requiring routine support.

Accepted limitation: a released/refunded installation that stays offline can retain its previously issued access. For annual access this lasts through the confirmed period; for perpetual access it can be indefinite. Immediate remote revocation cannot be promised.

Do not silently introduce mandatory periodic validation, a hardware fingerprint, or a stricter device limit to compensate.

## 7. Refund policy

- Voluntary full refund within fourteen days after first payment, for either paid offer.
- No additional blanket voluntary fourteen-day refund guarantee for automatic annual renewals.
- Mandatory customer rights remain separate and must be reflected in final legal terms.
- Do not add usage telemetry to decide refund eligibility.
- Immediate removal of already issued offline rights cannot be guaranteed.

Treatment of a later subscription-to-perpetual purchase under the "first payment" guarantee needs explicit wording before publication; the discovery did not settle that edge case.

## 8. Payment provider and seller

The owner reports being self-employed in Germany and already using Stripe for another project. No specific VAT regime, business classification, or tax registration is inferred.

Use a separate Stripe account for TypoMorph under the existing login. Provider account creation, verification, production keys, webhooks, tax configuration, and payment collection are not implemented or authorized by this documentation change.

Use provider-hosted payment collection so the application does not collect card numbers. No typed application content or keyboard events belong in payment, account, entitlement, support, or pricing requests.

Stripe replaces the legacy Lemon Squeezy integration. Stripe payments and TypoMorph's offline entitlement issuance are distinct responsibilities; having a Stripe account does not implement license validation.

Provider references for the later implementation review:
- [Multiple accounts](https://support.stripe.com/questions/create-and-manage-multiple-stripe-accounts)
- [Subscriptions](https://docs.stripe.com/api/subscriptions)
- [Currencies](https://docs.stripe.com/currencies)

Final tax/consumer-rights wording and the actual checkout configuration remain open release tasks, not assumptions supplied by this document.

## 9. Currency and public pricing

USD 7 and USD 19 are final product prices including applicable taxes. Tax and provider fees reduce seller proceeds. A customer's bank may independently apply currency conversion or fees.

The website may show an approximate EUR equivalent beside USD:
- include an approximation mark, rate date, and explanation that payment is in USD;
- periodically refresh a stored reference rate using website automation, without a dedicated currency service;
- show only USD if the rate is unavailable or stale;
- make no currency requests from the desktop application and do not infer location to choose a price.

The freshness cutoff and refresh schedule need operational definition. Other display currencies are deferred. No donation pricing, minimum-price subscription, or twelve-month update limitation is part of the final approved offer.

## 10. Support and implementation gap

Email or a contact form, no guaranteed response time, for trial and both paid offers. Publish common troubleshooting instructions; do not promise priority support or an SLA.

`crates/licensing` now verifies Ed25519 signatures using trusted keys supplied by
the application. The JSON envelope contains `key_id`, the exact UTF-8 JSON
`payload`, and a 128-character hexadecimal signature. The signature covers
`TypoMorph offline entitlement v1\0` followed by the exact payload bytes.
Envelope input is limited to 8 KiB; unknown fields and inconsistent claims fail
closed. Payload claims bind version 1, audience `typomorph-desktop`, random
account/installation IDs, plan, issuance/start times, and the confirmed expiry.
Trial expiry is exactly seven days after the original account-wide trial start.
Annual access ends at the signed paid-through date. Grace requires a separately
signed token issued after that date and ends exactly seven days later. Perpetual
grants reject expiry fields and remain valid offline indefinitely.

Six integration tests cover tampering, wrong account/device, unknown keys,
invalid signatures, trial sharing, expiry boundaries, perpetual access and grace.
Test signing secrets are deterministic fixtures only; no production key or
activation endpoint is embedded. The daemon does not yet consume entitlements.
This verifier does not implement email verification, three-device accounting,
Stripe/webhooks, cancellation/refunds, hostile local clock rollback, or recovery.
Legacy Lemon Squeezy/checksum code and CLI activation commands were removed.
Recognized developer-window exclusion is no longer gated by a paid tier.

Account retention/deletion rules, payment data inventory, final customer terms, activation-service recovery, and licensing tests are tracked in [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) and [TESTING.md](TESTING.md).
