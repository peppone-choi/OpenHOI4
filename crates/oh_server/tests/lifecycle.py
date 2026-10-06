"""Real executable startup/HTTP/active WS/Ctrl+C test on Windows, Linux, macOS.
Run from repository root AFTER npm ci/build and cargo build -p oh_server.
Windows sends an actual CTRL_C_EVENT into an isolated hidden child console;
it never signals the caller console or changes machine settings.
"""
import argparse
import base64
import json
import os
import queue
import threading
from pathlib import Path
import signal
import socket
import struct
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[3]

def recv_exact(sock, size):
    data = b""
    while len(data) < size:
        part = sock.recv(size - len(data))
        assert part, "unexpected socket EOF"
        data += part
    return data

def frame(sock):
    first, length = recv_exact(sock, 2)
    if length == 126:
        length = struct.unpack(">H", recv_exact(sock, 2))[0]
    elif length == 127:
        length = struct.unpack(">Q", recv_exact(sock, 8))[0]
    return first & 15, recv_exact(sock, length)

def send_binary(sock, data):
    mask = os.urandom(4)
    head = bytes([130, 128 | len(data)]) if len(data) < 126 else bytes([130, 254]) + struct.pack(">H", len(data))
    sock.sendall(head + mask + bytes(value ^ mask[i % 4] for i, value in enumerate(data)))

def ctrl_c(process):
    if sys.platform == "win32":
        helper = """import ctypes,sys,time
k=ctypes.windll.kernel32
k.FreeConsole()
assert k.AttachConsole(int(sys.argv[1])), ctypes.get_last_error()
assert k.SetConsoleCtrlHandler(None, True)
assert k.GenerateConsoleCtrlEvent(0, 0), ctypes.get_last_error()
time.sleep(.2)
k.FreeConsole()
"""
        subprocess.run([sys.executable, "-c", helper, str(process.pid)], check=True, timeout=5)
    else:
        process.send_signal(signal.SIGINT)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--open", action="store_true", help="also exercise the actual default browser launcher")
    parser.add_argument("--default", action="store_true", help="exercise default port and repository-relative data path")
    args = parser.parse_args()
    binary = ROOT / "target/debug" / ("oh_server.exe" if sys.platform == "win32" else "oh_server")
    fixtures = json.loads((ROOT / "target/wp05/wire-fixtures.json").read_text(encoding="utf-8"))
    with socket.socket() as temporary:
        temporary.bind(("127.0.0.1", 0)); port = temporary.getsockname()[1]
    if args.default: port = 8080
    kwargs = {}
    if sys.platform == "win32":
        startup = subprocess.STARTUPINFO()
        startup.dwFlags = subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = 0
        kwargs = {"creationflags": subprocess.CREATE_NEW_CONSOLE, "startupinfo": startup}
    # Run away from source assets: only executable and explicit data pack path.
    isolated = ROOT / "target/wp05/isolated"
    isolated.mkdir(parents=True, exist_ok=True)
    options = [] if args.default else ["--port", str(port), "--pack-root", str(ROOT / "data/packs/examples/m0")]
    process = subprocess.Popen([str(binary), *options, *(["--open"] if args.open else [])], cwd=ROOT if args.default else isolated, stdout=subprocess.PIPE, stderr=subprocess.PIPE, **kwargs)
    ws = None
    try:
        url = f"http://127.0.0.1:{port}/"
        deadline = time.monotonic() + 15
        while True:
            try:
                with urllib.request.urlopen(url, timeout=1) as response:
                    html = response.read(); assert response.status == 200 and b"/assets/" in html
                break
            except OSError:
                assert process.poll() is None, "server exited during startup"
                if time.monotonic() >= deadline: raise
                time.sleep(.05)
        ws = socket.create_connection(("127.0.0.1", port), timeout=5)
        key = base64.b64encode(os.urandom(16)).decode()
        ws.sendall(f"GET /ws HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n".encode())
        headers = b""
        while not headers.endswith(b"\r\n\r\n"): headers += recv_exact(ws, 1)
        assert b"101 Switching Protocols" in headers
        send_binary(ws, bytes(fixtures[0]["bytes"]))
        assert b"Welcome" in frame(ws)[1]
        send_binary(ws, bytes(fixtures[2]["bytes"]))
        assert b"Snapshot" in frame(ws)[1]
        startup_output = b""
        if args.open:
            while b"Default browser launcher exited successfully." not in startup_output:
                lines = queue.Queue()
                threading.Thread(target=lambda: lines.put(process.stdout.readline()), daemon=True).start()
                line = lines.get(timeout=10)
                assert line, "browser launcher exited without success evidence"
                startup_output += line
        ctrl_c(process)
        # Active websocket must receive a Close, not just an unhandled process kill.
        while frame(ws)[0] != 8: pass
        output, errors = process.communicate(timeout=10)
        output = startup_output + output
        print(output.decode(errors="replace")); print(errors.decode(errors="replace"))
        assert process.returncode == 0, process.returncode
        assert b"stopped normally" in output
        assert b"127.0.0.1" in output and b"Ctrl+C" in output
        if args.open:
            assert b"cannot open browser" not in errors and b"browser launcher failed" not in errors
        with socket.socket() as released:
            released.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            released.bind(("127.0.0.1", port))
        print("REQ-NET-02 HTTP built executable assets: PASS" if args.default else "REQ-NET-02 isolated executable assets: PASS")
        print("REQ-NET-06 actual CTRL_C_EVENT/SIGINT, active WS closure, exit 0, released port: PASS")
        if args.open: print("REQ-NET-06 actual default-browser launcher exit success: PASS (visual navigation not asserted)")
    finally:
        if ws: ws.close()
        if process.poll() is None:
            process.kill(); process.communicate()

if __name__ == "__main__":
    main()
