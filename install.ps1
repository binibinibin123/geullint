$ErrorActionPreference = "Stop"

$repository = "binibinibin123/geullint"
$architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
$target = switch ($architecture) {
    "X64" { "win32-x64" }
    "Arm64" { "win32-arm64" }
    default { throw "GeulLint: unsupported Windows architecture: $architecture" }
}

if ($env:GEULLINT_VERSION) {
    $version = $env:GEULLINT_VERSION.TrimStart("v")
}
else {
    $releases = Invoke-RestMethod `
        -Headers @{ "User-Agent" = "GeulLint-Installer" } `
        -Uri "https://api.github.com/repos/$repository/releases?per_page=1"
    $release = @($releases)[0]
    $version = ([string]$release.tag_name).TrimStart("v")
    if (-not $version) {
        throw "GeulLint: unable to determine the latest release."
    }
}

$archiveStem = "geullint-v$version-$target"
$archiveName = "$archiveStem.zip"
$downloadBase = "https://github.com/$repository/releases/download/v$version"
$installDirectory = if ($env:GEULLINT_INSTALL_DIR) {
    $env:GEULLINT_INSTALL_DIR
}
else {
    Join-Path $HOME ".local\bin"
}
$temporaryDirectory = Join-Path ([IO.Path]::GetTempPath()) ("geullint-" + [IO.Path]::GetRandomFileName())

New-Item -ItemType Directory -Path $temporaryDirectory | Out-Null
try {
    $archivePath = Join-Path $temporaryDirectory $archiveName
    $checksumPath = "$archivePath.sha256"
    Invoke-WebRequest -UseBasicParsing -Uri "$downloadBase/$archiveName" -OutFile $archivePath
    Invoke-WebRequest -UseBasicParsing -Uri "$downloadBase/$archiveName.sha256" -OutFile $checksumPath

    $expectedHash = ((Get-Content -LiteralPath $checksumPath -Raw).Trim() -split "\s+")[0].ToLowerInvariant()
    $actualHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $expectedHash) {
        throw "GeulLint: SHA-256 verification failed for $archiveName."
    }

    Expand-Archive -LiteralPath $archivePath -DestinationPath $temporaryDirectory -Force
    New-Item -ItemType Directory -Force -Path $installDirectory | Out-Null
    $source = Join-Path $temporaryDirectory "$archiveStem\geullint.exe"
    $destination = Join-Path $installDirectory "geullint.exe"
    Copy-Item -LiteralPath $source -Destination $destination -Force

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathEntries = @($userPath -split ";" | Where-Object { $_ })
    if ($installDirectory -notin $pathEntries) {
        $newPath = (@($pathEntries) + $installDirectory) -join ";"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    }
    if ($installDirectory -notin ($env:Path -split ";")) {
        $env:Path = "$installDirectory;$env:Path"
    }

    Write-Host ""
    Write-Host "GeulLint v$version installed at $destination"
    Write-Host "Run: geullint --version"
}
finally {
    if (Test-Path -LiteralPath $temporaryDirectory) {
        Remove-Item -LiteralPath $temporaryDirectory -Recurse -Force
    }
}
