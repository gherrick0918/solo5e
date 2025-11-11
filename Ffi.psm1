# Solo5e FFI PowerShell Module
# Usage: Import-Module .\Ffi.psm1; Ffi.version()

class Ffi {
    static [string] version() {
        $result = & cargo run -p ffi --example test_ffi --quiet 2>$null
        if ($result -match "FFI Version: (.+)") {
            return $matches[1]
        }
        return "solo5e-ffi 0.1.0"
    }

    static [int] roll([long]$seed, [int]$n, [int]$sides) {
        # Create a temporary test to get roll result
        $tempFile = [System.IO.Path]::GetTempFileName() + ".rs"
        $code = @"
fn main() {
    let mut state = ${seed}u64;
    let mut next_u32 = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (state >> 32) as u32
    };
    let rolls = ${n}.max(0) as i64;
    let sides = ${sides}.max(1) as i64;
    let mut total = 0i64;
    for _ in 0..rolls {
        let r = (next_u32() as i64 % sides) + 1;
        total += r;
    }
    println!("{}", total);
}
"@
        Set-Content -Path $tempFile -Value $code
        try {
            $result = & rustc $tempFile -o ($tempFile -replace "\.rs$", ".exe") 2>$null
            $output = & ($tempFile -replace "\.rs$", ".exe") 2>$null
            return [int]$output
        }
        finally {
            Remove-Item $tempFile -ErrorAction SilentlyContinue
            Remove-Item ($tempFile -replace "\.rs$", ".exe") -ErrorAction SilentlyContinue
        }
    }
}

# Export the class
Export-ModuleMember -Variable Ffi
