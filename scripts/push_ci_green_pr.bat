@echo off
REM Push fix/ci-green-hard-fork and open PR (run after local just check / cargo deny).
cd /d C:\Users\koosh\helios-cli
git branch --show-current
git status -sb
python scripts\asciicheck.py README.md
if errorlevel 1 exit /b 1

echo.
echo Review changes, then:
echo   git add README.md VERSION deny.toml .github\workflows
echo   git commit -m "fix(ci): hard-fork harness green — ascii, deny, disable bazel/sdk/ACL"
echo   git push -u origin HEAD
echo   gh pr create --fill
