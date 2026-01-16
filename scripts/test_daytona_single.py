#!/usr/bin/env python3
"""Test Daytona backend with a single card example."""

import asyncio
import logging
import os
import sys
import tempfile
import shutil
from pathlib import Path
from typing import Dict

# Enable logging
logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent))

# Read environment variables (do not hardcode secrets)
daytona_key = os.getenv("DAYTONA_API_KEY")
openai_key = os.getenv("OPENAI_API_KEY")
if not daytona_key or not openai_key:
    raise RuntimeError("Set DAYTONA_API_KEY and OPENAI_API_KEY in your environment.")

from src.lib.daytona_backend import DaytonaBackend, run_agent_in_daytona


async def test_provision_only():
    """Test just provisioning a sandbox (no agent run)."""
    print("=" * 60)
    print("Testing Daytona Backend - Provision Only")
    print("=" * 60)

    # Create a minimal test workspace
    with tempfile.TemporaryDirectory() as tmpdir:
        workspace = Path(tmpdir) / "test_workspace"
        workspace.mkdir()

        # Create a simple Rust project
        (workspace / "Cargo.toml").write_text("""
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
""")

        (workspace / "src").mkdir()
        (workspace / "src" / "lib.rs").write_text("""
/// Test function
pub fn hello() -> &'static str {
    "Hello from EngineBench!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from EngineBench!");
    }
}
""")

        print(f"Created test workspace at: {workspace}")

        backend = DaytonaBackend()

        def progress(msg: str):
            print(f"  > {msg}")

        try:
            print("\n1. Provisioning sandbox...")
            sandbox_id = await backend.provision(
                workspace,
                env_vars={"TEST_VAR": "test_value"},
                secrets={"OPENAI_API_KEY": os.environ["OPENAI_API_KEY"]},
                progress_callback=progress,
            )
            print(f"   Sandbox ID: {sandbox_id[:16]}...")

            print("\n2. Testing cargo build...")
            result = await backend.execute(
                sandbox_id,
                "cd /app/repo && cargo build",
                timeout=120,
            )
            print(f"   Build success: {result['success']}")
            if not result['success']:
                print(f"   Error: {result['stderr'][:500]}")

            print("\n3. Testing cargo test...")
            result = await backend.execute(
                sandbox_id,
                "cd /app/repo && cargo test",
                timeout=120,
            )
            print(f"   Test success: {result['success']}")
            print(f"   Output: {result['stdout'][:500]}")

            print("\n4. Destroying sandbox...")
            await backend.destroy(sandbox_id)
            print("   Done!")

            return True

        except Exception as e:
            print(f"\nError: {e}")
            import traceback
            traceback.print_exc()
            return False


async def test_with_overzealous():
    """Test with overzealous repo - minimal upload approach."""
    print("\n" + "=" * 60)
    print("Testing with Overzealous Repo (Minimal Upload)")
    print("=" * 60)

    # Get overzealous repo path
    overzealous_path = Path(os.path.expanduser("~/Documents/GitHub/overzealous"))
    if not overzealous_path.exists():
        print(f"Overzealous repo not found at {overzealous_path}")
        return False

    # Collect only essential files (Cargo files + source)
    print("Collecting essential files...")
    essential_files: Dict[str, bytes] = {}

    # Required crates for tcg_expansions
    required_crates = ["tcg_core", "tcg_rules_ex", "tcg_effects", "tcg_db", "tcg_expansions"]

    # Root Cargo files
    for filename in ["Cargo.toml", "Cargo.lock"]:
        filepath = overzealous_path / filename
        if filepath.exists():
            essential_files[filename] = filepath.read_bytes()

    # Collect each required crate
    for crate in required_crates:
        crate_path = overzealous_path / crate
        if not crate_path.exists():
            print(f"  Warning: {crate} not found")
            continue

        # Cargo.toml
        cargo_toml = crate_path / "Cargo.toml"
        if cargo_toml.exists():
            essential_files[f"{crate}/Cargo.toml"] = cargo_toml.read_bytes()

        # All .rs files in src/
        src_path = crate_path / "src"
        if src_path.exists():
            for rs_file in src_path.rglob("*.rs"):
                rel_path = str(rs_file.relative_to(overzealous_path))
                essential_files[rel_path] = rs_file.read_bytes()

    print(f"Collected {len(essential_files)} essential files")

    # Read the stub for df-009-pinsir
    stub_path = Path(__file__).parent.parent / "gold" / "stubs" / "df_009_pinsir.rs"
    if stub_path.exists():
        stub_content = stub_path.read_bytes()
        # Replace the pinsir file with stub
        essential_files["tcg_expansions/src/dragon_frontiers/df_009_pinsir.rs"] = stub_content
        print(f"Added stub: {stub_path.name}")

    backend = DaytonaBackend()

    def progress(msg: str):
        print(f"  > {msg}")

    try:
        print("\n1. Provisioning sandbox...")
        sandbox_id = await backend.provision(
            sandbox_dir=None,
            env_vars={"CARGO_TERM_COLOR": "never"},
            secrets={},
            overlay_files=essential_files,
            progress_callback=progress,
        )
        print(f"   Sandbox ID: {sandbox_id[:16]}...")

        print("\n2. Checking Rust version...")
        result = await backend.execute(
            sandbox_id,
            "rustc --version && cargo --version",
            timeout=30,
        )
        print(f"   {result['stdout'].strip()}")

        print("\n3. Listing workspace files...")
        result = await backend.execute(
            sandbox_id,
            "ls -la /app/repo && ls -la /app/repo/tcg_expansions/src/",
            timeout=30,
        )
        if result['stdout']:
            print(f"   {result['stdout'][:500]}")

        print("\n4. Running cargo check on tcg_expansions...")
        result = await backend.execute(
            sandbox_id,
            "cd /app/repo && cargo check --package tcg_expansions 2>&1 || echo 'CARGO CHECK FAILED'",
            timeout=300,
        )
        print(f"   Check success: {result['success']}")
        if result['stdout']:
            print(f"   Output:\n{result['stdout'][-2000:]}")

        print("\n5. Running eval tests for pinsir...")
        result = await backend.execute(
            sandbox_id,
            "cd /app/repo && cargo test --package tcg_expansions df_009_pinsir 2>&1 || echo 'TESTS FAILED'",
            timeout=120,
        )
        print(f"   Test success: {result['success']}")
        if result['stdout']:
            print(f"   Output:\n{result['stdout'][-1000:]}")

        print("\n6. Destroying sandbox...")
        await backend.destroy(sandbox_id)
        print("   Done!")

        return True

    except Exception as e:
        print(f"\nError: {e}")
        import traceback
        traceback.print_exc()
        return False


async def main():
    print("EngineBench Daytona Integration Test")
    print("=" * 60)

    # Test 1: Simple provision test
    success1 = await test_provision_only()

    if success1:
        print("\n✓ Simple provision test passed!")
    else:
        print("\n✗ Simple provision test failed!")
        return 1

    # Test 2: With actual overzealous repo
    success2 = await test_with_overzealous()

    if success2:
        print("\n✓ Overzealous repo test passed!")
    else:
        print("\n✗ Overzealous repo test failed!")
        return 1

    print("\n" + "=" * 60)
    print("All tests completed!")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)
