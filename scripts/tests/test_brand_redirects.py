"""The deployed host redirect preserves the installed updater's endpoint."""
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]


class BrandRedirectTests(unittest.TestCase):
    def test_old_host_pages_redirect_but_appcast_is_direct(self):
        config = json.loads((ROOT / 'site/vercel.json').read_text())
        redirect = config['redirects'][0]
        self.assertEqual(redirect['has'], [
            {'type': 'host', 'value': 'voxelite.worldwidewebb.co'}])
        self.assertTrue(redirect['permanent'])  # Vercel's permanent redirects are 308.
        self.assertEqual(redirect['destination'], 'https://quantum.worldwidewebb.co/:path')
        source = redirect['source']
        self.assertTrue(source.startswith('/:path(') and source.endswith(')'))
        pattern = re.compile('/' + source[len('/:path('):-1])
        self.assertIsNone(pattern.fullmatch('/appcast.xml'))
        for path in ['/', '/docs/', '/docs/mcp/', '/download', '/app', '/appcast.xml.example']:
            self.assertIsNotNone(pattern.fullmatch(path), path)
        for redirect in config['redirects'][1:]:
            self.assertNotEqual(redirect['source'], '/appcast.xml')

    def test_vercel_config_is_in_the_deployed_artifact(self):
        workflow = (ROOT / '.github/workflows/deploy-site.yml').read_text()
        self.assertIn('site/ "$RUNNER_TEMP/site-dist/"', workflow)
        self.assertNotIn('--exclude=vercel.json', workflow)
        self.assertNotIn('--exclude=/vercel.json', workflow)
        self.assertIn('cd site-dist', workflow)
        self.assertIn('vercel@', workflow)


if __name__ == '__main__':
    unittest.main()
