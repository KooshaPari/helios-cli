import importlib.util
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("verify_required_ci.py")
SPEC = importlib.util.spec_from_file_location("verify_required_ci", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
verify_required_ci = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(verify_required_ci)


class RequiredCiContractTest(unittest.TestCase):
    """Traces to: FR-HELIOS-CI-001 and FR-HELIOS-CI-002."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = Path(".github/workflows/rust-ci.yml").read_text(encoding="utf-8")
        cls.full_rust_workflow = Path(".github/workflows/rust-ci-full.yml").read_text(
            encoding="utf-8"
        )
        cls.full_rust_nextest_platform_workflow = Path(
            ".github/workflows/rust-ci-full-nextest-platform.yml"
        ).read_text(encoding="utf-8")
        cls.sdk_workflow = Path(".github/workflows/sdk.yml").read_text(encoding="utf-8")
        cls.blob_size_policy_workflow = Path(
            ".github/workflows/blob-size-policy.yml"
        ).read_text(encoding="utf-8")
        cls.npm_workflow = Path(".github/workflows/ci.yml").read_text(encoding="utf-8")

    def test_repository_workflow_satisfies_contract(self) -> None:
        self.assertEqual([], verify_required_ci.contract_errors(self.workflow))

    def test_missing_rust_ci_token_permissions_are_rejected(self) -> None:
        broken = self.workflow.replace("permissions:\n    contents: read\n\n", "", 1)
        self.assertIn(
            "Rust CI token permissions must be exactly contents: read",
            verify_required_ci.contract_errors(broken),
        )

    def test_broader_rust_ci_token_permissions_are_rejected(self) -> None:
        for replacement in (
            "permissions:\n    contents: write",
            "permissions:\n    contents: read\n    actions: read",
        ):
            with self.subTest(replacement=replacement):
                broken = self.workflow.replace("permissions:\n    contents: read", replacement, 1)
                self.assertIn(
                    "Rust CI token permissions must be exactly contents: read",
                    verify_required_ci.contract_errors(broken),
                )

    def test_rust_ci_least_privilege_is_documented(self) -> None:
        threat_model = Path("docs/security/threat-model.md").read_text(encoding="utf-8")
        self.assertIn("permissions:\n    contents: read", self.workflow)
        self.assertIn(
            "`rust-ci.yml` limits mutable pull-request jobs to `contents: read`",
            threat_model,
        )

    def test_full_rust_ci_permissions_are_least_privilege(self) -> None:
        self.assertEqual(
            [], verify_required_ci.full_rust_contract_errors(self.full_rust_workflow)
        )

    def test_full_rust_ci_broader_permissions_are_rejected(self) -> None:
        broken = self.full_rust_workflow.replace(
            "permissions:\n    contents: read",
            "permissions:\n    contents: read\n    actions: write",
            1,
        )
        self.assertIn(
            "Full Rust CI token permissions must be exactly contents: read",
            verify_required_ci.full_rust_contract_errors(broken),
        )

    def test_full_rust_ci_least_privilege_is_documented(self) -> None:
        threat_model = Path("docs/security/threat-model.md").read_text(encoding="utf-8")
        self.assertIn("permissions:\n    contents: read", self.full_rust_workflow)
        self.assertIn(
            "`rust-ci-full.yml` limits its repository token to `contents: read`",
            threat_model,
        )

    def test_full_rust_nextest_platform_permissions_are_least_privilege(self) -> None:
        self.assertEqual(
            [],
            verify_required_ci.full_rust_nextest_platform_contract_errors(
                self.full_rust_nextest_platform_workflow
            ),
        )

    def test_full_rust_nextest_platform_broader_permissions_are_rejected(self) -> None:
        broken = self.full_rust_nextest_platform_workflow.replace(
            "permissions:\n    contents: read",
            "permissions:\n    contents: read\n    actions: write",
            1,
        )
        self.assertIn(
            "Full Rust nextest platform token permissions must be exactly contents: read",
            verify_required_ci.full_rust_nextest_platform_contract_errors(broken),
        )

    def test_full_rust_nextest_platform_least_privilege_is_documented(self) -> None:
        threat_model = Path("docs/security/threat-model.md").read_text(encoding="utf-8")
        self.assertIn(
            "permissions:\n    contents: read", self.full_rust_nextest_platform_workflow
        )
        self.assertIn(
            "`rust-ci-full-nextest-platform.yml` limits its repository token to "
            "`contents: read`",
            threat_model,
        )

    def test_sdk_permissions_are_least_privilege(self) -> None:
        self.assertEqual([], verify_required_ci.sdk_contract_errors(self.sdk_workflow))

    def test_sdk_broader_permissions_are_rejected(self) -> None:
        broken = self.sdk_workflow.replace(
            "permissions:\n    contents: read",
            "permissions:\n    contents: read\n    actions: write",
            1,
        )
        self.assertIn(
            "SDK CI token permissions must be exactly contents: read",
            verify_required_ci.sdk_contract_errors(broken),
        )

    def test_sdk_least_privilege_is_documented(self) -> None:
        threat_model = Path("docs/security/threat-model.md").read_text(encoding="utf-8")
        self.assertIn("permissions:\n    contents: read", self.sdk_workflow)
        self.assertIn(
            "`sdk.yml` limits its repository token to `contents: read`", threat_model
        )

    def test_blob_size_policy_permissions_are_least_privilege(self) -> None:
        self.assertEqual(
            [],
            verify_required_ci.blob_size_policy_contract_errors(
                self.blob_size_policy_workflow
            ),
        )

    def test_blob_size_policy_broader_permissions_are_rejected(self) -> None:
        broken = self.blob_size_policy_workflow.replace(
            "permissions:\n    contents: read",
            "permissions:\n    contents: read\n    actions: write",
            1,
        )
        self.assertIn(
            "Blob size policy token permissions must be exactly contents: read",
            verify_required_ci.blob_size_policy_contract_errors(broken),
        )

    def test_blob_size_policy_least_privilege_is_documented(self) -> None:
        threat_model = Path("docs/security/threat-model.md").read_text(encoding="utf-8")
        self.assertIn(
            "permissions:\n    contents: read", self.blob_size_policy_workflow
        )
        self.assertIn(
            "`blob-size-policy.yml` limits its repository token to `contents: read`",
            threat_model,
        )

    def test_missing_mandatory_dependency_is_rejected(self) -> None:
        broken = self.workflow.replace("        workspace,\n", "", 1)
        self.assertIn(
            "results job does not depend on workspace",
            verify_required_ci.contract_errors(broken),
        )

    def test_unasserted_mandatory_result_is_rejected(self) -> None:
        broken = self.workflow.replace("needs.deny.result", "needs.deny.outcome")
        self.assertIn(
            "results job does not require deny to succeed",
            verify_required_ci.contract_errors(broken),
        )

    def test_repository_npm_workflow_satisfies_contract(self) -> None:
        self.assertEqual([], verify_required_ci.npm_contract_errors(self.npm_workflow))

    def test_swallowed_npm_staging_failure_is_rejected(self) -> None:
        broken = self.npm_workflow.replace(
            "              id: stage_npm_package\n",
            "              id: stage_npm_package\n              continue-on-error: true\n",
        )
        self.assertIn(
            "npm staging is allowed to fail",
            verify_required_ci.npm_contract_errors(broken),
        )

    def test_mutable_npm_install_is_rejected(self) -> None:
        broken = self.npm_workflow.replace("--frozen-lockfile", "--no-frozen-lockfile")
        self.assertIn(
            "npm CI does not enforce the committed lockfile",
            verify_required_ci.npm_contract_errors(broken),
        )

    def test_missing_npm_token_permissions_are_rejected(self) -> None:
        broken = self.npm_workflow.replace("permissions:\n    contents: read\n\n", "", 1)
        self.assertIn(
            "npm CI token permissions must be exactly contents: read",
            verify_required_ci.npm_contract_errors(broken),
        )

    def test_broader_npm_token_permissions_are_rejected(self) -> None:
        for replacement in (
            "permissions:\n    contents: write",
            "permissions:\n    contents: read\n    actions: read",
        ):
            with self.subTest(replacement=replacement):
                broken = self.npm_workflow.replace(
                    "permissions:\n    contents: read", replacement, 1
                )
                self.assertIn(
                    "npm CI token permissions must be exactly contents: read",
                    verify_required_ci.npm_contract_errors(broken),
                )

    def test_npm_staging_release_fallback_is_documented(self) -> None:
        threat_model = Path("docs/security/threat-model.md").read_text(encoding="utf-8")
        self.assertIn("CODEX_VERSION=0.115.0", self.npm_workflow)
        self.assertIn("GH_TOKEN: ${{ github.token }}", self.npm_workflow)
        self.assertIn("permissions:\n    contents: read", self.npm_workflow)
        self.assertIn("- [x] Staging failure propagation", threat_model)
        self.assertIn("- [x] Successful npm staging", threat_model)
        self.assertNotIn("- [ ] Successful npm staging", threat_model)
        normalized_threat_model = " ".join(threat_model.split())
        self.assertIn("exact seven public npm release assets", normalized_threat_model)
        self.assertIn("embedded package identity", threat_model)


if __name__ == "__main__":
    unittest.main()
