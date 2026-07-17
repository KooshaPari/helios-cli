@echo off
setlocal EnableExtensions
cd /d C:\Users\koosh\helios-cli
git add deny.toml .codespellrc
(
  echo fix^(ci^): ignore transitive rustsec advisories; skip codespell noise
  echo.
  echo Allow known git2/paste/serial advisories pulled via kla harness deps,
  echo and skip assets/fragemented/perf-results from codespell.
) > _commit_msg.txt
git commit -F _commit_msg.txt
set EC=%ERRORLEVEL%
del _commit_msg.txt 2>nul
if not %EC%==0 exit /b %EC%
git push
exit /b %ERRORLEVEL%
