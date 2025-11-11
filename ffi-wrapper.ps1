# PowerShell FFI Wrapper
# Usage: . .\ffi-wrapper.ps1; [Ffi]::version(); [Ffi]::roll(42, 3, 6)

class Ffi {
    static [string] version() {
        $output = cargo run -p cli -- ffi-version 2>$null
        return ($output | Where-Object { $_ -match "solo5e-ffi" }) -replace ".*Running.*", "" | ForEach-Object Trim
    }

    static [int] roll($seed, $n, $sides) {
        $output = cargo run -p cli -- ffi-roll --seed $seed --n $n --sides $sides 2>$null
        $number = ($output | Where-Object { $_ -match "^\d+$" }) | Select-Object -Last 1
        return [int]$number
    }
}

Write-Host "FFI wrapper loaded. Usage examples:" -ForegroundColor Green
Write-Host "  [Ffi]::version()" -ForegroundColor Cyan
Write-Host "  [Ffi]::roll(42, 3, 6)" -ForegroundColor Cyan
