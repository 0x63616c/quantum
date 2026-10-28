"""Keep Welcome capture fixtures inside their explicit disposable home."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "welcome_capture", ROOT / "scripts/prepare-welcome-capture.py"
)
CAPTURE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CAPTURE)


class WelcomeCaptureTests(unittest.TestCase):
    def test_both_revisions_use_real_projects_under_the_same_isolated_home(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            capture_home = directory / "home"
            histories = []
            for side in ("before", "after"):
                data = directory / side
                data.mkdir()
                CAPTURE.prepare(ROOT, capture_home, data)
                history = json.loads((data / "recents.json").read_text())
                histories.append(history)
                self.assertEqual(history["version"], 1)
                self.assertEqual(len(history["projects"]), 5)
                folders = []
                for project in history["projects"]:
                    path = Path(project["path"])
                    folders.append(str(path.parent.relative_to(capture_home.resolve())))
                    self.assertEqual(int(path.stat().st_mtime), project["modified"]["secs_since_epoch"])
                    source = "Autumn Workshop" if project["name"] == "Workshop Study" else project["name"]
                    source = ROOT / "examples" / (source.lower().replace(" ", "-") + ".voxl")
                    self.assertEqual(path.read_bytes(), source.read_bytes())
                self.assertEqual(folders, ["Quantum", "Quantum/client", "Desktop", "Quantum", "Quantum/samples"])
            self.assertEqual(histories[0], histories[1])
