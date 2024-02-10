# GitHub repository details
$version = "0.1.3"
$binName = "licensa"
$programName = "Licensa"
$architecture = "x86_64-windows"

$releaseVersion = "v${version}"
$releaseIdentifier = "${binName}-${releaseVersion}-${architecture}"
$releaseArchiveName = "${releaseIdentifier}.zip"

<#
.SYNOPSIS
    Writes a log message to the console with color-coded levels.

.DESCRIPTION
    This function writes a log message to the console with color-coded levels
    based on the specified severity level.
    
.PARAMETER Level
    The severity level of the log message. 
    
    Supported values are "info", "success", "warn", and "error".

.PARAMETER Message
    The log message to be written.
#>
function Write-Log {
    param(
        [string]$Level,
        [string]$Message,
        [bool]$NoNewLine = $false
    )

    switch ($Level) {
        "info" { $ForegroundColor = "Cyan" }
        "success" { $ForegroundColor = "Green" }
        "warn" { $ForegroundColor = "Yellow" }
        "error" { $ForegroundColor = "Red" }
        default { $ForegroundColor = "White" }
    }

    $TimeStamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $FormattedLevel = " [$($Level.ToUpper())] "

    Write-Host $TimeStamp -NoNewline
    Write-Host $FormattedLevel -NoNewline -ForegroundColor $ForegroundColor

    if ($NoNewLine) {
        Write-Host $Message -NoNewline
    }
    else {
        Write-Host $Message
    }
    
}

<#
.SYNOPSIS
    Retrieves the path to the user's Downloads directory.

.DESCRIPTION
    This function retrieves the path to the user's Downloads directory 
    by querying the registry. It returns the path to the Downloads directory.

.NOTES
    The Downloads directory path is retrieved from the registry key 
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders".
    
    The value corresponding to the GUID "{374DE290-123F-4565-9164-39C4925E467B}" 
    represents the Downloads directory.
#>
function Get-DownloadsDirectory {
    $shellFoldersPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders"
    $downloadsDirName = "{374DE290-123F-4565-9164-39C4925E467B}";
    $downloadsDir = Get-ItemPropertyValue -Path $shellFoldersPath -Name $downloadsDirName
    return $downloadsDir
}

function Get-GithubReleaseUrl {
    param (
        [string]$Version,
        [string]$ArchiveName
    )

    $baseUrl = "https://github.com/ekkolon/licensa/releases/download"
    $releaseUrl = "$baseUrl/$Version/$ArchiveName"
    return $releaseUrl
}

function Confirm-Installation {
    param (
        [string]$Version
    )

    $description = "This script is set to download and install Licensa CLI version $Version."
    Write-Host $description -ForegroundColor Yellow
    
    $question = "Are you sure you want to proceed? (y/N)"
    $confirmed = Read-Host $question
    
    if ($confirmed -ne 'Y' -or $confirmed -ne 'y') {
        Write-Log -Level "info" -Message "Installation process has been canceled"
        exit 0
    }
}

<#
.SYNOPSIS
    Downloads a binary archive from a specified URL.

.DESCRIPTION
    This function downloads a binary archive from the provided URL and saves it 
    to the specified output directory.

.PARAMETER Url
    The URL of the binary archive to download.

.PARAMETER OutDir
    The directory where the downloaded binary archive will be saved.
#>
function Get-BinaryArchive {
    param (
        [string]$Uri,
        [string]$OutFile
    )
    Write-Log -Level "info" -Message "Downloading assets from GitHub release page"
    try {
        Invoke-WebRequest -Uri $Uri -OutFile $OutFile
    }
    catch {
        Write-Log -Level "error" -Message $_
        exit 1
    }
}

<#
.SYNOPSIS
    Unpacks a binary archive to the specified destination path.

.DESCRIPTION
    This function unpacks a binary archive located at the given path 
    to the specified destination path.

.PARAMETER Path
    The path to the binary archive that you want to unpack.

.PARAMETER DestinationPath
    The path where the contents of the binary archive will be extracted.
#>
function Expand-BinaryArchive {
    param (
        [string]$Path,
        [string]$DestinationPath
    )
    try {
        Expand-Archive -Path $Path -DestinationPath $DestinationPath -Force
    }
    catch {
        Write-Log -Level "error" -Message $_
        exit 1
    }
}

<#
.SYNOPSIS
    Installs a binary executable to a specified destination path.

.DESCRIPTION
    This function installs a binary executable by moving the contents 
    of the specified source folder to the destination folder.
    
    If the destination folder already exists, it prompts the user 
    for confirmation before overwriting its contents.

.PARAMETER SourceFolder
    The path to the folder containing binary executable that you want to install.

.PARAMETER DestinationFolder
    The destination path where the binary executable will be installed/moved to
#>
function Install-Binary {
    param (
        [string]$Path,
        [string]$DestinationPath
    )

    Write-Log -Level "info" -Message "Installing Licensa executable"
    try {
        # Check if source folder exists
        if (-not (Test-Path -Path $Path -PathType Container)) {
            Write-Log -Level "error" -Message "Source folder $Path does not exist."
            exit 1
        }

        # Check if destination folder exists, create it if not
        if (-not (Test-Path -Path $DestinationPath -PathType Container)) {
            try {
                New-Item -ItemType Directory -Path $DestinationPath -ErrorAction Stop | Out-Null
            }
            catch {
                Write-Log -Level "error" -Message $_
                exit 1
            }
        } 
        else {
            # Prompt user for confirmation before overwriting destination folder
            $description = "Destination folder already exists: $DestinationPath"
            Write-Log -Level "warn" -Message $description
            
            $question = "Do you want to overwrite its contents? (y/N)"
            $confirmed = Read-Host $question
    
            if ($confirmed -eq "Y" -or $confirmed -eq "y") {
                try {
                    # Remove the existing destination folder
                    Remove-Item -Path $DestinationPath -Recurse -Force -ErrorAction Stop
                    New-Item -ItemType Directory -Path $DestinationPath -ErrorAction Stop | Out-Null
                    Write-Log -Level "info" -Message "Destination folder contents successfully removed"
                }
                catch {
                    Write-Log -Level "error" $_
                    exit 1
                }
            }
            else {
                Write-Log -Level "info" -Message "Installation process has been canceled"
                exit 0
            }
        }

        # Move contents from source folder to destination folder
        try {
            Get-ChildItem -Path $Path | Move-Item -Destination $DestinationPath -ErrorAction Stop
        }
        catch {
            Write-Log -Level "error" $_
            exit 1
        }
    }
    catch {
        # Error occurred during move operation
        Write-Log -Level "error" -Message $_
        exit 1
    }
}

<#
.SYNOPSIS
    Adds a specified folder to the PATH environment variable of the current 
    user's system if it doesn't exist.

.DESCRIPTION
    This function allows you to add a specified folder to the PATH environment variable.
    It prompts for confirmation before making any changes.

.PARAMETER Path
    The path to the folder that you want to add to the system variables.
#>
function Set-SystemVariable {
    param (
        [string]$Path
    )
    
    $question = "Would you like to add the Licensa CLI to your system variables? [y/N]";
    $confirmation = Read-Host $question
    if ($confirmation -eq 'Y' -or $confirmation -eq 'y') {    
        try {
            # Get the current user's PATH environment variable
            $userPath = [Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User)

            # Split the PATH variable into individual folders
            $pathFolders = $userPath -split ';'

            # Check if the folder is already in the PATH variable
            if ($pathFolders -contains $Path) {
                Write-Log -Level "info" -Message "Skipping setting environment variable because it's already set"
                return
            }
            else {
                # Add the folder to the PATH variable
                $newPath = $userPath + ";$Path"
                [System.Environment]::SetEnvironmentVariable('Path', $newPath, [System.EnvironmentVariableTarget]::User)
                Write-Log -Level "info" -Message "Licensa CLI added to system variables"
            }
        }
        catch {
            Write-Log -Level "error" -Message $_
        }
    }
}

# ================================================================== 
# Run script
# ================================================================== 

# Confirm installation
# ================================================================== 
Confirm-Installation -Version $version
Write-Host


# Download binary from GitHub release
# ================================================================== 
$downloadsDir = Get-DownloadsDirectory
$archiveDir = Join-Path $downloadsDir $releaseArchiveName
$releaseUrl = Get-GithubReleaseUrl -Version $releaseVersion -ArchiveName $releaseArchiveName
Get-BinaryArchive -Uri $releaseUrl -OutFile $archiveDir
Write-Log -Level "info" -Message "Archive saved to $archiveDir"


# Unpack the binary
# ================================================================== 
$sourceDir = Join-Path $downloadsDir $releaseIdentifier
Expand-BinaryArchive -Path $archiveDir -DestinationPath $downloadsDir
Write-Log -Level "info" -Message "Archive unpacked to $sourceDir"


# Install binary
# ================================================================== 
$outDir = Join-Path $env:Programfiles $programName
Install-Binary -Path $sourceDir -DestinationPath $outDir
Write-Log -Level "success" -Message "Licensa CLI has been installed successfully"


# Confirm and add Licensa CLI to system variables
# ================================================================== 
Set-SystemVariable -Path $outDir


# Cleanup downloaded and unpacked assets
# ================================================================== 
Remove-Item -Path $sourceDir
Remove-Item -Path $archiveDir


Write-Log -Level "success" -Message "Setup completed successfully"
Write-Host
