# Calculate version from git commit count (PowerShell version)

param(
    [switch]$ReturnVersionOnly = $false
)

try {
    # Get total commit count
    $commitCount = git rev-list --count HEAD
    
    # Calculate version components
    $major = [Math]::Floor($commitCount / 10000)
    $remaining = $commitCount % 10000
    $minor = [Math]::Floor($remaining / 1000)
    $remaining2 = $remaining % 1000
    $patch = [Math]::Floor($remaining2 / 100)
    $build = $remaining2 % 100
    
    # Format version string
    if ($major -gt 0) {
        $version = "${major}.${minor}.${patch}"
    } elseif ($minor -gt 0) {
        $version = "0.${minor}.${patch}"
    } elseif ($patch -gt 0) {
        $version = "0.0.${patch}"
    } else {
        $version = "0.0.0$build"
    }
    
    # Add v prefix for releases >= 1.0.0
    if ($major -ge 1) {
        $version = "v${version}"
    }
    
    if ($ReturnVersionOnly) {
        return $version
    } else {
        Write-Output $version
    }
} catch {
    Write-Error "Failed to calculate version: $_"
    exit 1
}