@echo off
cd /d C:\Users\koosh\helios-cli
gh pr create --title "fix(ci): hard-fork harness green - ascii, deny, disable bazel/sdk/ACL" --body-file _pr_body.md
exit /b %ERRORLEVEL%
