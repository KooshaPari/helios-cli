"""Direct behavioral contracts for :class:`harness.scratchpad.ScratchpadFileSystem`."""

import asyncio
import json

from harness.scratchpad import ScratchpadFileSystem


def test_write_read_pointer_and_stats(tmp_path):
    async def scenario():
        scratchpad = ScratchpadFileSystem("session-1", base_dir=tmp_path)

        entry_id = await scratchpad.write(
            "notes",
            "hello scratchpad",
            description="test notes",
            metadata={"kind": "text"},
        )

        assert await scratchpad.read(entry_id) == b"hello scratchpad"
        assert scratchpad.get_pointer(entry_id) == (
            f"[file_id:{entry_id}] - notes: test notes (16 bytes)"
        )
        assert scratchpad.get_stats() == {
            "session_id": "session-1",
            "entry_count": 1,
            "total_size_bytes": 16,
            "total_size_mb": 16 / 1024 / 1024,
        }
        assert scratchpad.list_entries()[0].metadata == {"kind": "text"}

    asyncio.run(scenario())


def test_binary_and_json_content_are_serialized_to_bytes(tmp_path):
    async def scenario():
        scratchpad = ScratchpadFileSystem("session-1", base_dir=tmp_path)

        binary_id = await scratchpad.write("payload", b"\x00\xff")
        json_id = await scratchpad.write("record", {"answer": 42})

        assert await scratchpad.read(binary_id) == b"\x00\xff"
        assert json.loads((await scratchpad.read(json_id)).decode("utf-8")) == {"answer": 42}
        assert scratchpad._entries[binary_id].path.suffix == ".bin"
        assert scratchpad._entries[json_id].path.suffix == ".json"

    asyncio.run(scenario())


def test_index_persists_across_restart(tmp_path):
    async def scenario():
        original = ScratchpadFileSystem("session-1", base_dir=tmp_path)
        entry_id = await original.write("saved", "survives restart", "persisted")

        restarted = ScratchpadFileSystem("session-1", base_dir=tmp_path)

        assert await restarted.read(entry_id) == b"survives restart"
        assert restarted.get_pointer(entry_id) == (
            f"[file_id:{entry_id}] - saved: persisted (16 bytes)"
        )

    asyncio.run(scenario())


def test_delete_clear_and_missing_entries(tmp_path):
    async def scenario():
        scratchpad = ScratchpadFileSystem("session-1", base_dir=tmp_path)
        first_id = await scratchpad.write("first", "one")
        second_id = await scratchpad.write("second", "two")

        assert await scratchpad.delete(first_id) is True
        assert await scratchpad.read(first_id) is None
        assert await scratchpad.delete(first_id) is False
        assert scratchpad.get_pointer(first_id) == f"file_id:{first_id} - Not found"
        assert await scratchpad.clear() == 1
        assert await scratchpad.read(second_id) is None
        assert scratchpad.get_stats()["entry_count"] == 0

    asyncio.run(scenario())
