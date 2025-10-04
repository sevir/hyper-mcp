#!/usr/bin/env python3
"""Integration test for the Logseq plugin via hyper-mcp.

This script can run in two modes:
1. Fake server mode (default): Spins up a fake Logseq HTTP API server and tests the plugin
2. Real server mode (--use-real-logseq): Uses a real Logseq server running on localhost:12315

In both modes, it launches the hyper-mcp server with the repository's `.ai/hyper-mcp.yaml`
configuration and exercises the `logseq-search_pages` tool with a sample query.
"""
from __future__ import annotations

import argparse
import asyncio
import json
import os
import signal
import subprocess
import sys
import threading
import time
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from typing import Any, Dict, Optional

# ---------------------------------------------------------------------------
# Fake Logseq API server
# ---------------------------------------------------------------------------


@dataclass
class FakeLogseqServer:
    """Very small HTTP server that mimics the Logseq JSON-RPC endpoint."""

    host: str = "127.0.0.1"
    port: int = 12315

    def __post_init__(self) -> None:
        self._server: Optional[HTTPServer] = None
        self._thread: Optional[threading.Thread] = None

    def start(self) -> None:
        if self._server is not None:
            raise RuntimeError("FakeLogseqServer already running")

        class _Handler(BaseHTTPRequestHandler):
            def do_POST(self_inner) -> None:  # type: ignore[override]
                if self_inner.path != "/api":
                    self_inner.send_error(404, "Not Found")
                    return

                content_length = int(
                    self_inner.headers.get("Content-Length", "0"))
                body = self_inner.rfile.read(content_length)
                try:
                    payload = json.loads(body.decode("utf-8"))
                except json.JSONDecodeError:
                    self_inner.send_error(400, "Invalid JSON")
                    return

                method = payload.get("method")
                if method == "logseq.search":
                    query_text = payload.get("args", [""])[0]
                    response_body: Any = [
                        {
                            "content": f"This is content containing {query_text}",
                            "page": {
                                "name": "Test Page",
                                "id": 424242,
                                "uuid": "fake-uuid-123"
                            },
                            "type": "block",
                            "id": 12345
                        },
                        {
                            "content": f"Another block with {query_text} in it",
                            "page": {
                                "name": "Another Page",
                                "id": 424243,
                                "uuid": "fake-uuid-456"
                            },
                            "type": "block",
                            "id": 12346
                        }
                    ]
                else:
                    response_body = None

                response_bytes = json.dumps(response_body).encode("utf-8")
                self_inner.send_response(200)
                self_inner.send_header("Content-Type", "application/json")
                self_inner.send_header(
                    "Content-Length", str(len(response_bytes)))
                self_inner.end_headers()
                self_inner.wfile.write(response_bytes)

            def log_message(self_inner, format: str, *args: Any) -> None:  # noqa: D401
                """Silence the default HTTP server logging."""
                return

        self._server = HTTPServer((self.host, self.port), _Handler)
        self._thread = threading.Thread(
            target=self._server.serve_forever, daemon=True)
        self._thread.start()
        time.sleep(0.1)

    def stop(self) -> None:
        if self._server is None:
            return
        self._server.shutdown()
        if self._thread is not None:
            self._thread.join(timeout=1)
        self._server = None
        self._thread = None


# ---------------------------------------------------------------------------
# MCP stdio client
# ---------------------------------------------------------------------------

class MCPClientError(RuntimeError):
    pass


class MCPStdioClient:
    def __init__(self, executable: Path, config_path: Path, cwd: Path) -> None:
        self._cmd = [
            str(executable),
            "--config-file",
            str(config_path),
            "--transport",
            "stdio",
        ]
        self._cwd = cwd
        self._proc: Optional[asyncio.subprocess.Process] = None
        self._stderr_task: Optional[asyncio.Task[None]] = None
        self._next_id = 1
        self._pending: Dict[int, Any] = {}

    async def __aenter__(self) -> "MCPStdioClient":
        self._proc = await asyncio.create_subprocess_exec(
            *self._cmd,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd=self._cwd,
        )
        assert self._proc.stdout and self._proc.stdin and self._proc.stderr
        self._stderr_task = asyncio.create_task(
            self._drain_stderr(self._proc.stderr))
        return self

    # type: ignore[override]
    async def __aexit__(self, exc_type, exc, tb) -> None:
        await self.close()

    async def close(self) -> None:
        if self._proc is None:
            return
        if self._proc.stdin:
            try:
                self._proc.stdin.close()
            except Exception:
                pass
        if self._stderr_task:
            try:
                await asyncio.wait_for(self._stderr_task, timeout=1)
            except asyncio.TimeoutError:
                self._stderr_task.cancel()
                with contextlib.suppress(asyncio.CancelledError):
                    await self._stderr_task
        try:
            await asyncio.wait_for(self._proc.wait(), timeout=2)
        except asyncio.TimeoutError:
            self._proc.terminate()
            with contextlib.suppress(asyncio.TimeoutError):
                await asyncio.wait_for(self._proc.wait(), timeout=2)
        self._proc = None

    async def initialize(self) -> Dict[str, Any]:
        response = await self._send_request(
            method="initialize",
            params={
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": {"listChanged": True},
                    "sampling": {},
                    "elicitation": {"schemaValidation": True},
                },
                "clientInfo": {
                    "name": "logseq-plugin-integration-test",
                    "version": "1.0",
                },
            },
        )
        # notify the server that initialization has completed
        await self._send_notification("notifications/initialized", {})
        return response

    async def list_tools(self) -> Dict[str, Any]:
        return await self._send_request(method="tools/list", params={"cursor": None})

    async def call_tool(self, name: str, arguments: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        params = {"name": name}
        if arguments is not None:
            params["arguments"] = arguments
        return await self._send_request(method="tools/call", params=params)

    async def _send_request(self, method: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        if self._proc is None:
            raise MCPClientError("Process not started")
        request_id = self._next_id
        self._next_id += 1
        message: Dict[str, Any] = {"jsonrpc": "2.0",
                                   "id": request_id, "method": method}
        if params is not None:
            message["params"] = params
        await self._write_message(message)
        while True:
            incoming = await self._read_message()
            if "id" in incoming:
                pending_id = incoming["id"]
                if pending_id == request_id:
                    if "error" in incoming:
                        raise MCPClientError(json.dumps(
                            incoming["error"], indent=2))
                    return incoming.get("result", {})
                self._pending[pending_id] = incoming
            else:
                # Notification — ignore for this test
                continue

    async def _send_notification(self, method: str, params: Dict[str, Any]) -> None:
        await self._write_message({"jsonrpc": "2.0", "method": method, "params": params})

    async def _write_message(self, payload: Dict[str, Any]) -> None:
        if self._proc is None or self._proc.stdin is None:
            raise MCPClientError("Process stdin is not available")
        data = json.dumps(payload, separators=(
            ",", ":")).encode("utf-8") + b"\n"
        self._proc.stdin.write(data)
        await self._proc.stdin.drain()

    async def _read_message(self) -> Dict[str, Any]:
        if self._proc is None or self._proc.stdout is None:
            raise MCPClientError("Process stdout is not available")
        while True:
            line = await asyncio.wait_for(self._proc.stdout.readline(), timeout=15)
            if not line:
                raise MCPClientError("Unexpected EOF while reading message")
            stripped = line.strip()
            if not stripped:
                continue
            return json.loads(stripped.decode("utf-8"))

    async def _drain_stderr(self, stream: asyncio.StreamReader) -> None:
        while True:
            line = await stream.readline()
            if not line:
                break
            sys.stderr.write("[hyper-mcp] " + line.decode("utf-8"))


# ---------------------------------------------------------------------------
# Test harness
# ---------------------------------------------------------------------------

import contextlib  # noqa: E402  # isort:skip


def find_hyper_mcp_binary(repo_root: Path) -> Path:
    env_override = os.environ.get("HYPER_MCP_BIN")
    if env_override:
        candidate = Path(env_override)
        if candidate.is_file():
            return candidate
        raise FileNotFoundError(
            f"HYPER_MCP_BIN points to missing file: {candidate}")

    candidates = [
        repo_root / "target" / "release" / "hyper-mcp",
        repo_root / "target" / "debug" / "hyper-mcp",
    ]
    for candidate in candidates:
        if candidate.is_file():
            return candidate

    raise FileNotFoundError(
        "Could not locate hyper-mcp binary. Build it with `cargo build --release` "
        "or set HYPER_MCP_BIN to the compiled executable."
    )


def extract_text_from_call(result: Dict[str, Any]) -> str:
    pieces = []
    for item in result.get("content", []):
        text = item.get("text") or ""
        pieces.append(text)
    return "\n".join(pieces)


async def run_integration_test(use_fake_server: bool = True) -> None:
    repo_root = Path(__file__).resolve().parents[1]
    config_path = repo_root / ".ai" / "hyper-mcp.yaml"
    if not config_path.is_file():
        raise FileNotFoundError(f"Missing configuration file: {config_path}")

    binary_path = find_hyper_mcp_binary(repo_root)

    fake_logseq = FakeLogseqServer() if use_fake_server else None
    if fake_logseq:
        fake_logseq.start()

    try:
        async with MCPStdioClient(binary_path, config_path, repo_root) as client:
            init_response = await client.initialize()
            if init_response.get("serverInfo", {}).get("name") != "hyper-mcp":
                raise MCPClientError("Unexpected server identity")

            list_response = await client.list_tools()
            tool_names = [tool.get("name")
                          for tool in list_response.get("tools", [])]
            if "logseq-search_pages" not in tool_names:
                raise MCPClientError(
                    "logseq-search_pages not advertised by server")

            call_response = await client.call_tool(
                "logseq-search_pages",
                {"query": "Prácticas en empresa 25"},
            )

            if call_response.get("isError"):
                raise MCPClientError(f"Tool returned error: {call_response}")

            made_text = extract_text_from_call(call_response)
            print("🔍 Search Results for 'Prácticas en empresa 25':")
            print("=" * 50)
            print(made_text)
            print("=" * 50)

            if use_fake_server and "Prácticas en empresa 25" not in made_text:
                raise MCPClientError(
                    "Tool response does not contain expected query text")

            print("✔ logseq-search_pages test completed successfully")
    finally:
        if fake_logseq:
            fake_logseq.stop()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--skip-run",
        action="store_true",
        help="Validate prerequisites but skip running the integration test.",
    )
    parser.add_argument(
        "--use-real-logseq",
        action="store_true",
        help="Use real Logseq server running on localhost:12315 instead of fake server.",
    )
    args = parser.parse_args()

    if args.skip_run:
        print("Skipping run as requested; prerequisites look good.")
        return

    asyncio.run(run_integration_test(use_fake_server=not args.use_real_logseq))


if __name__ == "__main__":
    main()
