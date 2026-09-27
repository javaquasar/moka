param(
    [int]$Operations = 20000,
    [int]$Repetitions = 5,
    [string]$OutputDirectory = ""
)

$ErrorActionPreference = "Stop"
$benchRoot = $PSScriptRoot
$repoRoot = (Resolve-Path (Join-Path $benchRoot "../..")).Path
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path $benchRoot "results"
}
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null

$manifest = Join-Path $benchRoot "Cargo.toml"
cargo build --release --manifest-path $manifest
$binary = Join-Path $benchRoot "target/release/moka-post-removal-observer-bench.exe"
$csv = Join-Path $OutputDirectory "raw.csv"
$metadata = Join-Path $OutputDirectory "metadata.txt"

"repetition,mode,operation,operations,gross_bytes,allocations,bytes_per_operation,allocations_per_operation" | Set-Content $csv
$configurations = @(
    @("off", "insert"),
    @("listener", "insert"),
    @("observer", "insert"),
    @("off", "remove"),
    @("listener", "remove"),
    @("observer", "remove")
)

for ($repetition = 1; $repetition -le $Repetitions; $repetition++) {
    for ($offset = 0; $offset -lt $configurations.Count; $offset++) {
        $configuration = $configurations[($offset + $repetition - 1) % $configurations.Count]
        $row = & $binary $configuration[0] $configuration[1] $Operations
        "$repetition,$row" | Add-Content $csv
    }
}

@(
    "source_revision=$(git -C $repoRoot rev-parse HEAD)",
    "source_dirty=$(-not [string]::IsNullOrWhiteSpace((git -C $repoRoot status --porcelain)))",
    "rustc=$(rustc --version --verbose | Out-String)",
    "os=$([System.Environment]::OSVersion.VersionString)",
    "processor=$env:PROCESSOR_IDENTIFIER",
    "operations=$Operations",
    "repetitions=$Repetitions"
) | Set-Content $metadata

Write-Output $csv
Write-Output $metadata
