import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import stage_npm_packages


EXPECTED_PACKAGES = [
    "codex",
    "codex-linux-x64",
    "codex-linux-arm64",
    "codex-darwin-x64",
    "codex-darwin-arm64",
    "codex-win32-x64",
    "codex-win32-arm64",
]


def npm_tarball(package: str, version: str) -> bytes:
    config = stage_npm_packages.CODEX_PLATFORM_PACKAGES.get(package)
    npm_tag = config.get("npm_tag") if config else None
    package_version = f"{version}-{npm_tag}" if npm_tag else version
    package_metadata = {"name": "@openai/codex", "version": package_version}
    if config:
        package_metadata.update({"os": [config["os"]], "cpu": [config["cpu"]]})
    else:
        package_metadata["optionalDependencies"] = {
            platform["npm_name"]: (f"npm:@openai/codex@{version}-{platform['npm_tag']}")
            for platform in stage_npm_packages.CODEX_PLATFORM_PACKAGES.values()
        }
    package_json = json.dumps(package_metadata).encode()
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:gz") as archive:
        info = tarfile.TarInfo("package/package.json")
        info.size = len(package_json)
        archive.addfile(info, io.BytesIO(package_json))
    return output.getvalue()


def release_payload(version: str = "0.115.0") -> tuple[dict, dict[str, bytes]]:
    payload = {
        "tag_name": f"rust-v{version}",
        "name": version,
        "draft": False,
        "prerelease": False,
        "assets": [],
    }
    downloads = {}
    for package in EXPECTED_PACKAGES:
        name = stage_npm_packages.tarball_name_for_package(package, version)
        content = npm_tarball(package, version)
        downloads[name] = content
        payload["assets"].append(
            {
                "name": name,
                "size": len(content),
                "digest": f"sha256:{hashlib.sha256(content).hexdigest()}",
                "browser_download_url": (
                    f"https://github.com/openai/codex/releases/download/rust-v{version}/{name}"
                ),
            }
        )
    return payload, downloads


class ResolveReleaseWorkflowTest(unittest.TestCase):
    @patch.object(stage_npm_packages.subprocess, "check_output")
    def test_queries_the_upstream_repository(self, check_output) -> None:
        check_output.return_value = '{"url":"https://example.test/run","headSha":"abc123"}'

        workflow = stage_npm_packages.resolve_release_workflow("0.115.0")

        self.assertEqual(workflow["headSha"], "abc123")
        check_output.assert_called_once_with(
            [
                "gh",
                "run",
                "list",
                "--repo",
                stage_npm_packages.GITHUB_REPO,
                "--branch",
                "rust-v0.115.0",
                "--json",
                "workflowName,url,headSha",
                "--workflow",
                stage_npm_packages.WORKFLOW_NAME,
                "--jq",
                "first(.[])",
            ],
            cwd=stage_npm_packages.REPO_ROOT,
            text=True,
        )


class ReleaseAssetsFallbackTest(unittest.TestCase):
    @patch.object(stage_npm_packages.urllib.request, "urlopen")
    def test_fetches_exact_release_tag_without_authorization(self, urlopen) -> None:
        payload, _downloads = release_payload()
        urlopen.return_value = io.BytesIO(json.dumps(payload).encode())

        self.assertEqual(payload, stage_npm_packages.fetch_release_metadata("0.115.0"))

        request = urlopen.call_args.args[0]
        self.assertEqual(
            "https://api.github.com/repos/openai/codex/releases/tags/rust-v0.115.0",
            request.full_url,
        )
        self.assertNotIn("Authorization", dict(request.header_items()))

    def test_accepts_exact_pinned_release_package_set(self) -> None:
        payload, _downloads = release_payload()

        assets = stage_npm_packages.validate_release_npm_assets(
            payload, "0.115.0", EXPECTED_PACKAGES
        )

        self.assertEqual(7, len(assets))
        self.assertEqual(
            {stage_npm_packages.tarball_name_for_package(p, "0.115.0") for p in EXPECTED_PACKAGES},
            {asset.name for asset in assets},
        )

    def test_rejects_missing_or_extra_npm_asset(self) -> None:
        payload, _downloads = release_payload()
        extra_asset = {
            **payload["assets"][0],
            "name": "codex-npm-freebsd-x64-0.115.0.tgz",
        }
        for assets in (payload["assets"][:-1], [*payload["assets"], extra_asset]):
            with self.subTest(asset_count=len(assets)):
                broken = {**payload, "assets": assets}
                with self.assertRaisesRegex(RuntimeError, "exact npm release asset set"):
                    stage_npm_packages.validate_release_npm_assets(
                        broken, "0.115.0", EXPECTED_PACKAGES
                    )

    def test_rejects_missing_digest_or_wrong_release_identity(self) -> None:
        payload, _downloads = release_payload()
        no_digest = {
            **payload,
            "assets": [
                {**payload["assets"][0], "digest": None},
                *payload["assets"][1:],
            ],
        }
        with self.assertRaisesRegex(RuntimeError, "SHA-256 digest"):
            stage_npm_packages.validate_release_npm_assets(no_digest, "0.115.0", EXPECTED_PACKAGES)
        with self.assertRaisesRegex(RuntimeError, "release identity"):
            stage_npm_packages.validate_release_npm_assets(
                {**payload, "tag_name": "rust-v0.116.0"},
                "0.115.0",
                EXPECTED_PACKAGES,
            )

    def test_rejects_unapproved_version_or_package_set(self) -> None:
        for version, packages in (
            ("0.116.0", EXPECTED_PACKAGES),
            ("0.115.0", EXPECTED_PACKAGES[:-1]),
        ):
            with self.subTest(version=version, packages=packages):
                with self.assertRaisesRegex(RuntimeError, "fallback is restricted"):
                    stage_npm_packages.require_release_fallback_scope(version, packages)

    def test_download_rejects_digest_mismatch(self) -> None:
        payload, _downloads = release_payload()
        asset = stage_npm_packages.validate_release_npm_assets(
            payload, "0.115.0", EXPECTED_PACKAGES
        )[0]
        with tempfile.TemporaryDirectory() as temp_dir:
            with patch.object(
                stage_npm_packages,
                "open_public_url",
                return_value=io.BytesIO(b"x" * asset.size_in_bytes),
            ):
                with self.assertRaisesRegex(RuntimeError, "SHA-256 mismatch"):
                    stage_npm_packages.download_release_asset(asset, Path(temp_dir))

    @patch.object(stage_npm_packages.subprocess, "check_call")
    def test_workflow_download_failure_is_classified_for_fallback(self, check_call) -> None:
        check_call.side_effect = stage_npm_packages.subprocess.CalledProcessError(1, ["gh"])
        artifact = stage_npm_packages.WorkflowArtifact("linux", 10)
        with tempfile.TemporaryDirectory() as temp_dir:
            with self.assertRaises(stage_npm_packages.WorkflowArtifactsUnavailable):
                stage_npm_packages.download_artifacts("123", Path(temp_dir), [artifact])

    def test_rejects_embedded_package_version_mismatch(self) -> None:
        content = npm_tarball("codex-linux-x64", "9.9.9")
        with tempfile.TemporaryDirectory() as temp_dir:
            tarball = Path(temp_dir) / "codex-npm-linux-x64-0.115.0.tgz"
            tarball.write_bytes(content)
            with self.assertRaisesRegex(RuntimeError, "package identity mismatch"):
                stage_npm_packages.validate_npm_tarball(tarball, "codex-linux-x64", "0.115.0")

    def test_stages_all_seven_verified_assets_from_public_urls(self) -> None:
        payload, downloads = release_payload()

        def public_url(url: str):
            return io.BytesIO(downloads[url.rsplit("/", 1)[-1]])

        with tempfile.TemporaryDirectory() as temp_dir:
            with (
                patch.object(stage_npm_packages, "fetch_release_metadata", return_value=payload),
                patch.object(stage_npm_packages, "open_public_url", side_effect=public_url),
            ):
                paths = stage_npm_packages.stage_from_release_assets(
                    "0.115.0", EXPECTED_PACKAGES, Path(temp_dir)
                )

            self.assertEqual(7, len(paths))
            self.assertTrue(all(path.is_file() for path in paths))


if __name__ == "__main__":
    unittest.main()
