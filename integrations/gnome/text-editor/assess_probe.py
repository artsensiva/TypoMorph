#!/usr/bin/env python3
"""Interpret bounded, content-free test markers; does not launch or certify an app."""
import argparse
from enum import IntEnum
import sys

MAX_OUTPUT_BYTES = 64 * 1024


class Outcome(IntEnum):
    OBSERVED = 0
    FAILED = 1
    KNOWN_UNSAFE = 2


# Fixed facts the subclass-boundary case may report. Order and values are pinned in
# EXPECTED below; the full set exists so the runner can display an unexpected value.
SUBCLASS_FACTS = (
    'TYPE_IS_SOURCE_BUFFER_SUBCLASS', 'TYPE_IS_NOT_SOURCE_BUFFER_SUBCLASS',
    'TAGS_PRESENT', 'TAGS_ABSENT',
    'LEGACY_EDIT_HANDLERS_PRESENT', 'LEGACY_EDIT_HANDLERS_ABSENT',
    'LIBRARY_API_ABSENT', 'SPLICE_REFUSED_UNCHANGED', 'SPLICE_NOT_REFUSED_OR_STATE_CHANGED',
    'ORDINARY_EDIT_NOTIFIES_TITLE', 'ORDINARY_EDIT_DOES_NOT_NOTIFY_TITLE',
)

EXPECTED = {
    'normal': [
        'typomorph-probe: WAITING',
        'typomorph-probe: RANGE_PAIR_ATTEMPTED',
        'typomorph-check: RANGE_AND_ORDINARY_UNDO_REDO_OBSERVED',
    ],
    'callback-begin-edit': [
        'typomorph-probe: WAITING',
        'typomorph-probe: CALLBACK_EDIT_INJECTED',
        'typomorph-probe: REFUSED',
        'typomorph-check: PREDELETE_REFUSAL_OBSERVED',
    ],
    'callback-delete-edit': [
        'typomorph-probe: WAITING',
        'typomorph-probe: CALLBACK_EDIT_INJECTED',
        'typomorph-probe: FAULT_PARTIAL_EDIT',
        'typomorph-check: KNOWN_UNSAFE_PARTIAL_EDIT_OBSERVED',
    ],
    # Observed 3 of 3 times on the real EditorDocument, then pinned. A change in any
    # fact (for example tags becoming absent) fails the case so it is re-examined.
    'subclass-boundary': [
        'typomorph-subclass: TYPE_IS_SOURCE_BUFFER_SUBCLASS',
        'typomorph-subclass: TAGS_PRESENT',
        'typomorph-subclass: LEGACY_EDIT_HANDLERS_PRESENT',
        'typomorph-subclass: SPLICE_REFUSED_UNCHANGED',
        'typomorph-subclass: ORDINARY_EDIT_NOTIFIES_TITLE',
    ],
    'gap-edit': [
        'typomorph-probe: WAITING',
        'typomorph-probe: FAULT_PARTIAL_EDIT',
        'typomorph-check: KNOWN_UNSAFE_PARTIAL_EDIT_OBSERVED',
    ],
}


def assess(case: str, output: bytes, app_exit_code: int) -> Outcome:
    """Require one complete ordered case and clean exit; never echo child output."""
    if case not in EXPECTED or len(output) > MAX_OUTPUT_BYTES or app_exit_code != 0:
        return Outcome.FAILED
    try:
        lines = output.decode('utf-8', errors='strict').splitlines()
    except UnicodeError:
        return Outcome.FAILED
    # Other app output cannot supply evidence. Any unknown/duplicate/reordered
    # protocol marker makes the trace invalid rather than being silently skipped.
    markers = [line for line in lines
               if line.startswith(('typomorph-probe:', 'typomorph-check:', 'typomorph-subclass:'))]
    if markers != EXPECTED[case]:
        return Outcome.FAILED
    if case in ('gap-edit', 'callback-delete-edit'):
        return Outcome.KNOWN_UNSAFE
    return Outcome.OBSERVED


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--case', choices=EXPECTED, required=True)
    parser.add_argument('--app-exit-code', type=int, required=True,
                        help='actual completed child status, supplied by the isolated runner')
    args = parser.parse_args()
    outcome = assess(args.case, sys.stdin.buffer.read(MAX_OUTPUT_BYTES + 1), args.app_exit_code)
    print({
        Outcome.OBSERVED: 'Expected synthetic observation recorded; production safety NOT certified.',
        Outcome.FAILED: 'Probe evidence incomplete, stale, mismatched, or process failed.',
        Outcome.KNOWN_UNSAFE: 'Known unsafe partial edit reproduced; safety requirement FAILED.',
    }[outcome])
    return int(outcome)


if __name__ == '__main__':
    raise SystemExit(main())
