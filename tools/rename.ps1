param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$DisplayName,
    [Parameter(Mandatory = $true)][string]$Identifier,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

$reserved = 'core', 'std', 'alloc', 'test', 'gpui', 'proc-macro', 'self', 'crate', 'super'
if ($Name -notmatch '^[a-z][a-z0-9]*(-[a-z0-9]+)*$') {
    throw "Name must be kebab-case, e.g. acme-notes: '$Name'"
}
if ($reserved -contains $Name) {
    throw "Name '$Name' clashes with a Rust or GPUI crate name"
}
if ([string]::IsNullOrWhiteSpace($DisplayName) -or $DisplayName.Contains('"') -or $DisplayName.Contains('\')) {
    throw 'DisplayName must not be empty and must not contain " or \'
}
if ($Identifier -notmatch '^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)+$') {
    throw "Identifier must be a reverse-DNS name, e.g. com.example.acmenotes: '$Identifier'"
}

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$dirty = git status --porcelain
if ($LASTEXITCODE -ne 0) { throw 'git status failed' }
if ($dirty) { throw 'The working tree must be clean. Commit or stash first.' }

$snake = $Name.Replace('-', '_')
$upper = $snake.ToUpperInvariant()
$replacements = @(
    @('dev.myapp.app', $Identifier),
    @('MYAPP_', ($upper + '_')),
    @('myapp_', ($snake + '_')),
    @('myapp=', ($snake + '=')),
    @('MyApp', $DisplayName),
    @('myapp', $Name)
)

$binary = '.png', '.ico', '.ttf', '.otf', '.bmp', '.jpg'
$skipped = 'tools/rename.ps1', 'docs/TEMPLATE.md'
$files = git ls-files | Where-Object {
    ($skipped -notcontains $_) -and ($binary -notcontains [IO.Path]::GetExtension($_).ToLowerInvariant())
}
if ($LASTEXITCODE -ne 0) { throw 'git ls-files failed' }

$utf8 = New-Object Text.UTF8Encoding $false
$moves = @()
$moves += , @('apps/myapp', "apps/$Name")
Get-ChildItem -Path 'crates' -Directory -Filter 'myapp-*' | ForEach-Object {
    $moves += , @("crates/$($_.Name)", "crates/$Name-$($_.Name.Substring('myapp-'.Length))")
}

foreach ($file in $files) {
    $path = Join-Path $root $file
    $text = [IO.File]::ReadAllText($path)
    $updated = $text
    $count = 0
    foreach ($pair in $replacements) {
        $count += ($updated.Length - $updated.Replace($pair[0], '').Length) / $pair[0].Length
        $updated = $updated.Replace($pair[0], $pair[1])
    }
    if ($file -eq 'README.md') {
        $updated = [regex]::Replace($updated, '(?s)<!-- template:start -->.*?<!-- template:end -->\r?\n(\r?\n)?', ("# " + $DisplayName.Replace('$', '$$') + "`n`n"))
    }
    if ($updated -ne $text) {
        if ($DryRun) {
            Write-Host ("{0,4} replacements  {1}" -f $count, $file)
        } else {
            [IO.File]::WriteAllText($path, $updated, $utf8)
        }
    }
}

if ($DryRun) {
    foreach ($move in $moves) { Write-Host ("move  {0} -> {1}" -f $move[0], $move[1]) }
    Write-Host 'Dry run: nothing was changed.'
    exit 0
}

foreach ($move in $moves) {
    git mv $move[0] $move[1]
    if ($LASTEXITCODE -ne 0) { throw "git mv $($move[0]) failed" }
}

cargo metadata --format-version 1 --offline > $null
if ($LASTEXITCODE -ne 0) {
    cargo metadata --format-version 1 > $null
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed; Cargo.lock was not refreshed' }
}

git rm -q docs/TEMPLATE.md tools/rename.ps1
if ($LASTEXITCODE -ne 0) { throw 'git rm failed' }

Write-Host ''
Write-Host "Renamed to $Name ($DisplayName, $Identifier)."
Write-Host 'Next: run ./tools/check.ps1, update the licence holder in LICENSE, then commit.'
