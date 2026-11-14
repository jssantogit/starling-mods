"""Source-package wiring checks (Rust logic is tested by Cargo in CI)."""
import json
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MOD = ROOT / 'mods' / 'badoptimizations' / 'native'
STARLING_REV = '09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2'
MIXINS_REV = 'ddd9700b21997c95d95ac6ee5ecafe3830cf622a'


class BadOptimizationsWiringTests(unittest.TestCase):
    def test_native_metadata_matches_game_and_bundle(self):
        info = json.loads((MOD / 'horizon-native.json').read_text())
        spec = tomllib.loads((MOD / 'mixins/badoptimizations/horizon-mixin.toml').read_text())
        self.assertEqual(info['horizon'], STARLING_REV)
        self.assertEqual(info['mixins'], MIXINS_REV)
        self.assertEqual(info['game'], '26.3')
        self.assertEqual(info['id'], 'badoptimizations')
        self.assertEqual(spec['mod']['id'], info['id'])
        self.assertEqual(spec['mod']['version'], info['version'])
        self.assertEqual(spec['compatibility']['horizon_revision'], STARLING_REV)

    def test_target_and_thread_local_cache_declared(self):
        spec = tomllib.loads((MOD / 'mixins/badoptimizations/horizon-mixin.toml').read_text())
        self.assertNotIn('state', spec, 'async Renderer::new cannot be a mixin state constructor')
        injection = spec['injection']
        self.assertEqual([x['kind'] for x in injection], ['HEAD', 'RETURN'])
        self.assertTrue(injection[0]['cancellable'])
        self.assertEqual({x['target'] for x in injection}, {'horizon_client_render::gfx::Renderer::update_lightmap'})
        self.assertTrue({x['helper'] for x in injection}.issubset(set(p.name for p in (MOD / 'mixins/badoptimizations').iterdir())))

    def test_package_is_readable_by_repository_packager(self):
        from tools.package_hnmod import package
        import tempfile
        import zipfile
        with tempfile.TemporaryDirectory() as folder:
            out = package(MOD, Path(folder) / 'badoptimizations.hnmod')
            with zipfile.ZipFile(out) as z:
                self.assertIn('horizon-native.json', z.namelist())
                self.assertIn('mixins/badoptimizations/runtime.rs', z.namelist())
                self.assertIn('mixins/badoptimizations/horizon-mixin.toml', z.namelist())

if __name__ == '__main__':
    unittest.main()
