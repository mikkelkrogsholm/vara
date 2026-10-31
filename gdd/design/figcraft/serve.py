#!/usr/bin/env python3
"""Serve current UI and the same design assets used by the Tauri build."""
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
class Handler(SimpleHTTPRequestHandler):
    def translate_path(self, path):
        path = path.split('?', 1)[0]
        if path.startswith('/ui/design/'):
            return str(ROOT / 'gdd/design' / path.removeprefix('/ui/design/'))
        return str(ROOT / 'desktop' / path.lstrip('/'))
HTTPServer(('127.0.0.1', 18770), Handler).serve_forever()
