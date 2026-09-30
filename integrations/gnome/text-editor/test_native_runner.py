"""No GUI/input test: check runner isolation and process cleanup helpers only."""
import os
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
from run_native import private_environment, stop, safe_markers


class RunnerTests(unittest.TestCase):
    def test_host_display_bus_and_personal_environment_are_not_inherited(self):
        with patch.dict(os.environ, {
            'DISPLAY': ':0', 'WAYLAND_DISPLAY': 'wayland-0',
            'DBUS_SESSION_BUS_ADDRESS': 'unix:path=/run/user/1000/bus',
            'GTK_IM_MODULE': 'host-input', 'HOME': '/personal',
            'SECRET_TEST_MARKER': 'do not inherit',
        }):
            env = private_environment(Path('/tmp/typomorph-wayland-fixture'))
        for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'WAYLAND_SOCKET',
                    'DBUS_SESSION_BUS_ADDRESS', 'DBUS_SYSTEM_BUS_ADDRESS',
                    'GTK_IM_MODULE', 'HOME', 'SECRET_TEST_MARKER'):
            self.assertNotIn(key, env)
        for key in ('XDG_RUNTIME_DIR', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME',
                    'XDG_DATA_HOME', 'XDG_STATE_HOME'):
            self.assertTrue(env[key].startswith('/tmp/typomorph-wayland-fixture'))
        self.assertEqual(env['GSETTINGS_BACKEND'], 'memory')

    def test_only_exact_content_free_markers_can_be_printed(self):
        output = (b'PRIVATE document text\n'
                  b'typomorph-probe: SECRET\n'
                  b'typomorph-harness: READY trailing PRIVATE\n'
                  b'typomorph-harness: READY\n'
                  b'typomorph-check: CHECK_FAILED\n')
        self.assertEqual(safe_markers(output), [
            'typomorph-harness: READY', 'typomorph-check: CHECK_FAILED'])

    def test_cleanup_reaps_running_child(self):
        child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(20)'])
        try:
            stop(child)
            self.assertIsNotNone(child.returncode)
            stop(child)
            stop(None)
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()

    def test_cleanup_kills_child_that_does_not_terminate(self):
        from unittest.mock import Mock
        child = Mock()
        child.poll.return_value = None
        child.wait.side_effect = [subprocess.TimeoutExpired('fixture', 3), 0]
        stop(child)
        child.terminate.assert_called_once()
        child.kill.assert_called_once()
        self.assertEqual(child.wait.call_count, 2)


if __name__ == '__main__':
    unittest.main()
