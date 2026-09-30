# Distribution implementation status

Updated 2026-09-28. No package was published, installed, signed or deployed during
this work. Version 1 is not release-ready.

The Linux installer now requires `--install`, a stable v1+ release, Debian/Ubuntu
amd64, and an installed `cosign`. It checks an exact GitHub repository/workflow/tag
identity and the GitHub Actions OIDC issuer against the checksum's Sigstore bundle,
then validates the selected package hash. Failure stops before elevation. It has
no unsigned, source-build, checksum-only or user-directory fallback. It never
starts the application or enables autostart. Existing v0 artifacts are refused.

This follows [Sigstore's blob verification interface](https://docs.sigstore.dev/cosign/verifying/verify/).
The installer itself must also be obtained through a trusted distribution channel;
artifact verification cannot secure an installer already modified by an attacker.

The release workflow runs workspace checks, produces the checksum bundle, and
creates a **draft** release for owner review. New Debian packaging omits the old
udev access rules and post-install device reload. Existing installed permissions
are not changed. The legacy deployment script now requires TLS certificate
verification; deployment credential handling and archive assembly still need work.

Five installer tests use fake curl/cosign/sudo commands and synthetic artifacts.
They verify argument binding, ordering, refusal on bad signature/hash/metadata,
and absence of automatic startup. They do **not** prove real signing, trust-root
availability, package installation, upgrade behavior, or native compatibility.
Run `python3 -m unittest discover -s scripts/tests -v` as a non-root user.

Remaining release work includes real signed-artifact verification, trusted updater
integration and rollback policy, release versioning, full package/install/uninstall
tests, Windows signing, macOS notarization, and platform beta evidence. On
2026-10-01 the landing-page source was updated to development status and approved
planned pricing, without purchase links or downloads. This source update is not
a website deployment or release.
