import unittest
from unittest.mock import patch

import stage_npm_packages


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


if __name__ == "__main__":
    unittest.main()
