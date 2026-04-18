param(
    [string]$IP,
    [Parameter(ValueFromRemainingArguments)]$ExtraArgs
)

$FTP_PORT = 5000
$NRO = "target/aarch64-skyline-switch/release/libssbu_easy_ewgf.nro"
$REMOTE = "/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libssbu_easy_ewgf.nro"
$DISABLED = $REMOTE -replace '/plugins/', '/disabled_plugins/'

if (-not $IP) {
    $ipFile = "$env:USERPROFILE\.switch\ip_addr.txt"
    if (Test-Path $ipFile) { $IP = (Get-Content $ipFile -Raw).Trim() }
}
if (-not $IP) {
    Write-Host "Usage: .\deploy.ps1 [-IP <switch-ip>]" -ForegroundColor Red
    exit 1
}

# resolve FTP credentials from ~/.switch/ftp_auth.txt (user:pass format)
$FTP_CRED = ""
$authFile = "$env:USERPROFILE\.switch\ftp_auth.txt"
if (Test-Path $authFile) {
    $FTP_CRED = (Get-Content $authFile -Raw).Trim()
} else {
    Write-Host "FTP auth not configured." -ForegroundColor Yellow
    $ans = Read-Host "Does your sys-ftpd require authentication? (y/n)"
    if ($ans -eq 'y') {
        $u = Read-Host "FTP user"
        $p = Read-Host "FTP password"
        $FTP_CRED = "${u}:${p}"
        Set-Content $authFile $FTP_CRED
        Write-Host "Saved to $authFile" -ForegroundColor Green
    } else {
        Set-Content $authFile ""
        Write-Host "Saved (no auth). Delete $authFile to reconfigure." -ForegroundColor Green
    }
}

cargo skyline build --release
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "Clearing disabled_plugins copy..." -ForegroundColor Cyan
if ($FTP_CRED) {
    curl.exe --silent --show-error "ftp://${IP}:${FTP_PORT}/" -Q "DELE ${DISABLED}" --ftp-pasv -u $FTP_CRED 2>$null
} else {
    curl.exe --silent --show-error "ftp://${IP}:${FTP_PORT}/" -Q "DELE ${DISABLED}" --ftp-pasv 2>$null
}
$LASTEXITCODE = 0

Write-Host "Uploading to $IP..." -ForegroundColor Cyan
if ($FTP_CRED) {
    curl.exe -T $NRO "ftp://${IP}:${FTP_PORT}${REMOTE}" --ftp-pasv -u $FTP_CRED
} else {
    curl.exe -T $NRO "ftp://${IP}:${FTP_PORT}${REMOTE}" --ftp-pasv
}
if ($LASTEXITCODE -ne 0) {
    Write-Host "FTP upload failed." -ForegroundColor Red
    exit 1
}

Write-Host "Done. Restart the game." -ForegroundColor Green
