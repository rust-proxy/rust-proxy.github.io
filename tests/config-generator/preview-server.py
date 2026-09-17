"""Local preview of the assembled site with the same layout as GitHub Pages."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[2]


class Handler(SimpleHTTPRequestHandler):
    def log_message(self, format, *args):
        pass


port = int(sys.argv[1]) if len(sys.argv) > 1 else 8765
server = ThreadingHTTPServer(('127.0.0.1', port), partial(Handler, directory=str(root / 'site')))
print(f'Preview: http://127.0.0.1:{port}/', flush=True)
server.serve_forever()
