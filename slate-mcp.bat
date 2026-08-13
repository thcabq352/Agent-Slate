@echo off
REM Stdio MCP launcher for Hermes / Claude Code (blocking slate_film_factory).
setlocal
cd /d "%~dp0"
if not defined SLATE_PACKS_DIR set "SLATE_PACKS_DIR=%~dp0workflows\packs"
if not defined SLATE_DATA_DIR set "SLATE_DATA_DIR=%USERPROFILE%\Documents\Slate"
set "ENGINE=%~dp0target\debug\slate-engine.exe"
if not exist "%ENGINE%" set "ENGINE=%~dp0target\release\slate-engine.exe"
if not exist "%ENGINE%" (
  echo slate-engine.exe not found. Run: cargo build -p slate-engine
  exit /b 1
)
"%ENGINE%" mcp %*
