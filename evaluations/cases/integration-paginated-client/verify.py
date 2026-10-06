import importlib.util
import json
import socket
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit


def main(root):
    checks = []

    def check(name, passed, detail):
        checks.append({"id": name, "status": "passed" if passed else "failed", "detail": detail})

    source = root / "client.py"
    if not source.is_file():
        return [{"id": "module", "status": "failed", "detail": "client.py missing"}]
    try:
        spec = importlib.util.spec_from_file_location("case_client", source)
        client = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(client)
    except Exception as exc:
        return [{"id": "module", "status": "failed", "detail": f"Import failed: {type(exc).__name__}: {str(exc)[:100]}"}]

    observed = {"auth": [], "posts": [], "reads": [], "names": []}
    refuse_later_page = [False]
    token = "fixture-token-47"

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def reply(self, status, value):
            encoded = json.dumps(value).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(encoded)))
            self.end_headers()
            self.wfile.write(encoded)

        def authorized(self):
            observed["auth"].append(self.headers.get("Authorization"))
            if self.headers.get("Authorization") != f"Bearer {token}":
                self.reply(401, {"error": "unauthorized"})
                return False
            return True

        def do_GET(self):
            if not self.authorized():
                return
            url = urlsplit(self.path)
            cursor = parse_qs(url.query).get("cursor", [None])[0]
            observed["reads"].append((url.path, cursor))
            if url.path != "/items":
                return self.reply(404, {"error": "missing"})
            if cursor is None:
                return self.reply(200, {"items": [{"id": "a", "name": "Amber"}, {"id": "b", "name": "Bay"}], "next_cursor": "next page/2"})
            if cursor == "next page/2":
                if refuse_later_page[0]:
                    return self.reply(403, {"error": "forbidden"})
                return self.reply(200, {"items": [{"id": "c", "name": "Cove"}], "next_cursor": None})
            return self.reply(400, {"error": "bad cursor"})

        def do_POST(self):
            if not self.authorized():
                return
            observed["posts"].append(self.path)
            if self.path != "/items" or self.headers.get("Content-Type", "").split(";", 1)[0] != "application/json":
                return self.reply(400, {"error": "bad request"})
            try:
                length = int(self.headers.get("Content-Length", "0"))
                data = json.loads(self.rfile.read(min(length, 4096)))
                name = data["name"]
            except Exception:
                return self.reply(400, {"error": "bad json"})
            observed["names"].append(name)
            if name == "Refused Cove":
                return self.reply(403, {"error": "forbidden"})
            if name == "Uncertain Cove":
                self.close_connection = True
                try:
                    self.connection.shutdown(socket.SHUT_RDWR)
                except OSError:
                    pass
                self.connection.close()
                return
            return self.reply(201, {"id": "new-1", "name": name})

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    server.daemon_threads = True
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    try:
        try:
            items = client.list_all(base, token)
            check("pagination", items == [{"id": "a", "name": "Amber"}, {"id": "b", "name": "Bay"}, {"id": "c", "name": "Cove"}] and observed["reads"] == [("/items", None), ("/items", "next page/2")], "Read all pages in order with encoded cursor")
        except Exception as exc:
            check("pagination", False, f"Read raised {type(exc).__name__}")
        try:
            created = client.create_item(base, token, "New Cove")
            check("confirmed-write", created == {"id": "new-1", "name": "New Cove"} and observed["names"].count("New Cove") == 1, "Confirmed write returns server record once")
        except Exception as exc:
            check("confirmed-write", False, f"Write raised {type(exc).__name__}")
        try:
            client.create_item(base, token, "Uncertain Cove")
            uncertain = False
        except Exception as exc:
            uncertain = isinstance(exc, client.MutationUncertain)
        check("uncertain-write", uncertain and observed["names"].count("Uncertain Cove") == 1, "Uncertain response raises MutationUncertain with exactly one POST")
        try:
            client.create_item(base, token, "Refused Cove")
            refused = False
        except Exception as exc:
            refused = not isinstance(exc, client.MutationUncertain)
        check("refused-write", refused and observed["names"].count("Refused Cove") == 1,
              "Explicit HTTP refusal raises a different exception without replay")
        refuse_later_page[0] = True
        try:
            client.list_all(base, token)
            rejected_partial = False
        except Exception:
            rejected_partial = True
        check("failed-later-page", rejected_partial,
              "A later-page refusal raises rather than returning partial retrieval")
        check("authorization", len(observed["auth"]) >= 4 and all(value == f"Bearer {token}" for value in observed["auth"]), "Every observed request carried the required bearer token")
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)
    return checks


if __name__ == "__main__":
    try:
        result = main(Path(sys.argv[1]).resolve()) if len(sys.argv) == 2 else [{"id": "usage", "status": "failed", "detail": "Expected workspace path"}]
    except Exception as exc:
        result = [{"id": "verifier-error", "status": "failed", "detail": f"{type(exc).__name__}: {str(exc)[:120]}"}]
    print(json.dumps({"checks": result}, separators=(",", ":")))
    sys.exit(0 if all(item["status"] == "passed" for item in result) else 1)
