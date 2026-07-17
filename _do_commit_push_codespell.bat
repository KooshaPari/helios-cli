@echo off
setlocal EnableExtensions
cd /d C:\Users\koosh\helios-cli
git add .github/workflows/codespell.yml .codespellrc
(
  echo fix^(ci^): load .codespellrc in Codespell workflow
  echo.
  echo Action was ignoring the config file; wire config input and skip
  echo research/fragemented/mojo noise paths.
) > _commit_msg.txt
git commit -F _commit_msg.txt
set EC=%ERRORLEVEL%
del _commit_msg.txt 2>nul
if not %EC%==0 exit /b %EC%
git push
exit /b %ERRORLEVEL%
