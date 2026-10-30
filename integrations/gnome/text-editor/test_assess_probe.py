"""Synthetic protocol tests only. No GTK, C probe or application is executed."""
import unittest
from assess_probe import assess, Outcome, MAX_OUTPUT_BYTES


class AssessmentTests(unittest.TestCase):
    NORMAL = b'typomorph-probe: WAITING\ntypomorph-probe: RANGE_PAIR_ATTEMPTED\ntypomorph-check: RANGE_AND_ORDINARY_UNDO_REDO_OBSERVED\n'
    def test_complete_normal_observation_requires_clean_process_exit(self):
        self.assertEqual(assess('normal', self.NORMAL, 0), Outcome.OBSERVED)
        for status in (-11, 1, 124):
            self.assertEqual(assess('normal', self.NORMAL, status), Outcome.FAILED)

    def test_predelete_refusal_is_separate_from_normal_correction(self):
        trace = b'typomorph-probe: WAITING\ntypomorph-probe: CALLBACK_EDIT_INJECTED\ntypomorph-probe: REFUSED\ntypomorph-check: PREDELETE_REFUSAL_OBSERVED\n'
        self.assertEqual(assess('callback-begin-edit', trace, 0), Outcome.OBSERVED)
        self.assertEqual(assess('normal', trace, 0), Outcome.FAILED)

    def test_partial_edits_never_report_success(self):
        for case, callback in [('gap-edit', b''), ('callback-delete-edit', b'typomorph-probe: CALLBACK_EDIT_INJECTED\n')]:
            trace = b'typomorph-probe: WAITING\n' + callback + b'typomorph-probe: FAULT_PARTIAL_EDIT\ntypomorph-check: KNOWN_UNSAFE_PARTIAL_EDIT_OBSERVED\n'
            self.assertEqual(assess(case, trace, 0), Outcome.KNOWN_UNSAFE)
            self.assertNotEqual(assess(case, trace, 0), 0)

    def test_missing_duplicate_reordered_and_stale_markers_fail(self):
        lines = self.NORMAL.splitlines(keepends=True)
        for trace in [b''.join(lines[:-1]), self.NORMAL * 2,
                      b''.join(reversed(lines)), self.NORMAL + b'typomorph-check: NOT_READY\n',
                      b'typomorph-probe: STALE\n' + self.NORMAL]:
            self.assertEqual(assess('normal', trace, 0), Outcome.FAILED)

    SUBCLASS = (b'typomorph-subclass: TYPE_IS_SOURCE_BUFFER_SUBCLASS\ntypomorph-subclass: TAGS_PRESENT\n'
                b'typomorph-subclass: LEGACY_EDIT_HANDLERS_PRESENT\ntypomorph-subclass: SPLICE_REFUSED_UNCHANGED\n'
                b'typomorph-subclass: ORDINARY_EDIT_NOTIFIES_TITLE\n')

    def test_subclass_boundary_requires_every_pinned_fact_and_a_refusal(self):
        self.assertEqual(assess('subclass-boundary', self.SUBCLASS, 0), Outcome.OBSERVED)
        # An accepted or state-changing splice, an absent library API and a changed
        # guard fact must each fail, never be reported as an observation.
        for old, new in [(b'SPLICE_REFUSED_UNCHANGED', b'SPLICE_NOT_REFUSED_OR_STATE_CHANGED'),
                         (b'TAGS_PRESENT', b'TAGS_ABSENT'),
                         (b'LEGACY_EDIT_HANDLERS_PRESENT', b'LEGACY_EDIT_HANDLERS_ABSENT'),
                         (b'SPLICE_REFUSED_UNCHANGED', b'LIBRARY_API_ABSENT')]:
            self.assertEqual(assess('subclass-boundary', self.SUBCLASS.replace(old, new), 0), Outcome.FAILED)
        self.assertEqual(assess('subclass-boundary', self.SUBCLASS, 1), Outcome.FAILED)
        self.assertEqual(assess('subclass-boundary', self.SUBCLASS.replace(
            b'typomorph-subclass: TAGS_PRESENT\n', b''), 0), Outcome.FAILED)

    def test_oversize_invalid_encoding_and_unknown_case_fail(self):
        for case, trace in [('normal', b'x' * (MAX_OUTPUT_BYTES + 1)),
                            ('normal', self.NORMAL + b'\xff'), ('other', self.NORMAL)]:
            self.assertEqual(assess(case, trace, 0), Outcome.FAILED)

    def test_unrelated_output_is_not_used_as_evidence(self):
        self.assertEqual(assess('normal', b'OTHER OUTPUT\n', 0), Outcome.FAILED)
        self.assertEqual(assess('normal', b'OTHER OUTPUT\n' + self.NORMAL, 0), Outcome.OBSERVED)


if __name__ == '__main__':
    unittest.main()
