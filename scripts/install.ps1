
# Logging helper
function Write-Log {
    param(
        [string]$Level,
        [string]$Message
    )

    switch ($Level) {
        "info" { $ForegroundColor = "Blue" }
        "success" { $ForegroundColor = "Green" }
        "warn" { $ForegroundColor = "Yellow" }
        "error" { $ForegroundColor = "Red" }
        default { $ForegroundColor = "White" }
    }

    $TimeStamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $FormattedLevel = "[$($Level.ToUpper())]"
    $FormattedMessage = "$TimeStamp $FormattedLevel $Message"

    Write-Host $FormattedMessage -ForegroundColor $ForegroundColor
}

# GitHub repository details
$version = "0.1.0"
$binName = "licensa"
$releaseTag = "v${version}"

$assetNameUnpacked = "${binName}-${releaseTag}-x86_64-windows"
$assetNameZip = "${assetNameUnpacked}.zip"

$releaseDownloadUrl = "https://github.com/ekkolon/licensa/releases/download/$releaseTag/$assetNameZip"

# Target download directory
$userDownloadsFolder = Join-Path $env:USERPROFILE "Downloads"
$downloadPath = Join-Path $userDownloadsFolder $assetNameZip
# TODO: handle errors for `downloadPath`

# Prompt the user for confirmation
$confirmation = Read-Host "This script is set to download and install Licensa CLI ${releaseTag}.
Are you sure you want to proceed? (Type 'Y' for Yes, 'N' for No)"

if ($confirmation -ne 'Y' -or $confirmation -ne 'y') {
    Write-Log -Level "info" -Message "The installation process has been canceled"
    Exit
}

# Download binary from GitHub release
Write-Log -Level "info" -Message "Downloading assets from GitHub ..."
Invoke-WebRequest -Uri $releaseDownloadUrl -OutFile $downloadPath
Write-Log -Level "success" -Message "Download succeded!"

# Unpack the binary
Write-Log -Level "info" -Message "Unpacking..."
Expand-Archive -Path $downloadPath -DestinationPath $userDownloadsFolder -Force
Write-Log -Level "success" -Message "Successfully unpacked ${assetNameZip}"

Write-Log -Level "info" -Message "Installing..."
$programFiles = $env:Programfiles
$destinationPath = Join-Path $programFiles "Licensa"

# Check if the program folder exists
if (-not (Test-Path -Path $destinationPath -PathType Container)) {
    # Create the program folder if it doesn't exist
    New-Item -ItemType Directory -Path $destinationPath | Out-Null
    Write-Log -Level "info" -Message "Licensa CLI directory created at: $destinationPath"
}
else {
    # TODO: improve error handling
    # TODO: Suggest override
    Write-Host "Licensa CLI is already installed at: $destinationPath"
    Exit
}

$sourceFolder = Join-Path $downloadsFolder $assetNameUnpacked

# Move unpacked source to destination
Get-ChildItem -Path $sourceFolder | Move-Item -Destination $destinationPath

# Remove downloaded zip and unpacked directory
Remove-Item -Path $sourceFolder
Remove-Item -Path $downloadPath

# Confirm adding Licensa CLI to system variables
$confirmation = Read-Host "Would you like to add the Licensa CLI to your system variables? [y/N]"
if ($confirmation -eq 'Y' -or $confirmation -eq 'y') {    
    # Add directory path to system environment variable
    $envPath = [System.Environment]::GetEnvironmentVariable('Path', [System.EnvironmentVariableTarget]::User)
    $newPath = $envPath + ";" + $destinationPath
    [System.Environment]::SetEnvironmentVariable('Path', $newPath, [System.EnvironmentVariableTarget]::User)
    Write-Log -Level "info" -Message "Added Licensa CLI to system variables"
}

Write-Log -Level "success" -Message "Licensa CLI has been installed successfully"

exit


