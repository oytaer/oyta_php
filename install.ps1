param(
    [string]$InstallDir = "$env:LOCALAPPDATA\oyta"
)

$ErrorActionPreference = "Stop"

$BinaryName = "oyta.exe"
$BinaryInZip = "oyta-windows-x64.exe"
$DownloadUrl = "https://github.com/user-attachments/files/27195352/oyta-windows-x64.zip"

function Write-Header {
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host "       Oyta PHP Installer" -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host ""
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Add-ToPath {
    param([string]$PathToAdd)
    
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$PathToAdd*") {
        [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$PathToAdd", "User")
        Write-Host "Added $PathToAdd to user PATH" -ForegroundColor Green
    } else {
        Write-Host "$PathToAdd is already in PATH" -ForegroundColor Yellow
    }
}

Write-Header

Write-Host "Detected architecture: x86_64" -ForegroundColor White
Write-Host "Install directory: $InstallDir" -ForegroundColor White
Write-Host ""

$tempDir = Join-Path $env:TEMP "oyta-install-$(Get-Random)"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
$zipFile = Join-Path $tempDir "oyta.zip"

try {
    Write-Host "Downloading..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $zipFile -UseBasicParsing
    
    Write-Host "Extracting..." -ForegroundColor Yellow
    Expand-Archive -Path $zipFile -DestinationPath $tempDir -Force
    
    $extractedBinary = Join-Path $tempDir $BinaryInZip
    if (-not (Test-Path $extractedBinary)) {
        Write-Host "Error: Binary not found after extraction (expected: $BinaryInZip)" -ForegroundColor Red
        Write-Host "Contents of temp dir:" -ForegroundColor Yellow
        Get-ChildItem -Path $tempDir
        exit 1
    }
    
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Write-Host "Created install directory: $InstallDir" -ForegroundColor Green
    }
    
    $destBinary = Join-Path $InstallDir $BinaryName
    Move-Item -Path $extractedBinary -Destination $destBinary -Force
    Write-Host "Installed binary to: $destBinary" -ForegroundColor Green
    
    Add-ToPath -PathToAdd $InstallDir
    
    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Green
    Write-Host "Installation completed successfully!" -ForegroundColor Green
    Write-Host "Binary installed to: $destBinary" -ForegroundColor Green
    Write-Host ""
    Write-Host "Please restart your terminal or run:" -ForegroundColor Yellow
    Write-Host "  `$env:PATH += ';$InstallDir'" -ForegroundColor White
    Write-Host ""
    Write-Host "Run 'oyta --help' to get started" -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Green
    
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
} finally {
    if (Test-Path $tempDir) {
        Remove-Item -Path $tempDir -Recurse -Force
    }
}
