# Agent-Slate — Windows from-source setup (PowerShell 5.1+ / 7).
# Usage:
#   powershell -ExecutionPolicy Bypass -File .\install.ps1
#   powershell -ExecutionPolicy Bypass -File .\install.ps1 -Grok -Engine
#
# Clones the repo if you are not already inside it. Does not store API keys.

[CmdletBinding()]
param(
  [switch]$Grok,
  [switch]$Cursor,
  [switch]$Engine,
  [switch]$SkipFfmpeg,
  [string]$RepoUrl = 'https://github.com/thcabq352/Agent-Slate.git',
  [string]$Dest = ''
)

$ErrorActionPreference = 'Stop'

function Test-Cmd($Name) {
  return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Ensure-Node {
  if (Test-Cmd 'node') {
    $major = [int]((node -v) -replace '[^0-9].*', '')
    if ($major -ge 20) { return }
    Write-Host "Node $((node -v)) is too old — need 20+." -ForegroundColor Yellow
  }
  if (-not (Test-Cmd 'winget')) {
    throw 'Node 20+ is required. Install from https://nodejs.org or winget (App Installer).'
  }
  Write-Host '→ installing Node.js LTS via winget'
  winget install --id OpenJS.NodeJS.LTS -e --accept-package-agreements --accept-source-agreements
  $env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')
  if (-not (Test-Cmd 'node')) {
    throw 'Node was installed — close this window and run install.ps1 again so PATH updates.'
  }
}

function Ensure-Repo {
  $here = Get-Location
  if ((Test-Path (Join-Path $here 'package.json')) -and (Select-String -Path (Join-Path $here 'package.json') -Pattern '"name": "agent-slate"' -Quiet)) {
    return $here.Path
  }
  if (-not $Dest) { $Dest = Join-Path $env:USERPROFILE 'Agent-Slate' }
  if (-not (Test-Path (Join-Path $Dest 'package.json'))) {
    if (-not (Test-Cmd 'git')) { throw 'git is required to clone Agent-Slate.' }
    Write-Host "→ cloning $RepoUrl to $Dest"
    git clone $RepoUrl $Dest
  }
  Set-Location $Dest
  return $Dest
}

Write-Host "Agent-Slate Windows setup"
Ensure-Node
$root = Ensure-Repo
Set-Location $root

$setupArgs = @()
if ($Grok) { $setupArgs += '--grok' }
if ($Cursor) { $setupArgs += '--cursor' }
if ($Engine) { $setupArgs += '--engine' }
if ($SkipFfmpeg) { $setupArgs += '--skip-ffmpeg' }
else { $setupArgs += '--ffmpeg' }

node (Join-Path $root 'scripts\setup.mjs') @setupArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
