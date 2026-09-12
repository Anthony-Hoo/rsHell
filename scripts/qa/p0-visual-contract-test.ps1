param(
    [string]$ReplayReport,
    [string]$ReplayRoot,
    [ValidateSet("Windows", "NonWindows")]
    [string]$ReplayPlatform
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-VisualContractValidator {
    param([Parameter(Mandatory)][bool]$PlatformIsWindows)

    $harnessPath = Join-Path $PSScriptRoot "p0-smoke.ps1"
    $tokens = $null
    $parseErrors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile(
        $harnessPath,
        [ref]$tokens,
        [ref]$parseErrors
    )
    if ($parseErrors.Count -ne 0) {
        throw "p0-smoke.ps1 contains PowerShell parse errors."
    }
    $matches = @($ast.FindAll({
                param($node)
                $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                $node.Name -ceq "Assert-VisualContract"
            }, $true))
    if ($matches.Count -ne 1) {
        throw "Expected exactly one Assert-VisualContract function."
    }

    $body = $matches[0].Body.Extent.Text
    $body = $body.Substring(1, $body.Length - 2)
    $predicate = 'if ($IsWindows)'
    if ([regex]::Matches($body, [regex]::Escape($predicate)).Count -ne 1) {
        throw "Assert-VisualContract must contain one platform predicate."
    }
    $script:VisualContractTestIsWindows = $PlatformIsWindows
    $body = $body.Replace($predicate, 'if ($script:VisualContractTestIsWindows)')
    return [scriptblock]::Create($body)
}

function Get-ExpectedMatrix {
    param([Parameter(Mandatory)][bool]$PlatformIsWindows)

    $matrix = @(
        @("compact-empty", "empty", 800, 600, "compact"),
        @("compact-twenty-tabs", "twenty_tabs", 800, 600, "compact"),
        @("compact-grid", "grid", 800, 600, "compact"),
        @("compact-editor", "editor", 800, 600, "compact"),
        @("compact-settings", "settings", 800, 600, "compact"),
        @("compact-import", "import", 800, 600, "compact"),
        @("compact-recovery", "recovery", 800, 600, "compact"),
        @("standard-connected", "connected", 1360, 860, "standard"),
        @("standard-single", "single", 1360, 860, "standard"),
        @("standard-h-split", "h_split", 1360, 860, "standard"),
        @("standard-v-split", "v_split", 1360, 860, "standard"),
        @("standard-top-bottom-3", "top_bottom_3", 1360, 860, "standard"),
        @("standard-grid", "grid", 1360, 860, "standard"),
        @("standard-editor", "editor", 1360, 860, "standard"),
        @("standard-settings", "settings", 1360, 860, "standard"),
        @("standard-import", "import", 1360, 860, "standard"),
        @("standard-host-key", "host_key", 1360, 860, "standard"),
        @("standard-authentication", "authentication", 1360, 860, "standard"),
        @("standard-failure", "failure", 1360, 860, "standard"),
        @("standard-recovery", "recovery", 1360, 860, "standard")
    )
    if ($PlatformIsWindows) {
        $matrix += @(
            @("windows-standard-connected", "connected", 1000, 700, "standard"),
            @("windows-standard-twenty-tabs", "twenty_tabs", 1000, 700, "standard"),
            @("windows-standard-grid", "grid", 1000, 700, "standard"),
            @("windows-standard-editor", "editor", 1000, 700, "standard"),
            @("windows-standard-settings", "settings", 1000, 700, "standard"),
            @("windows-standard-import", "import", 1000, 700, "standard")
        )
    }
    else {
        $matrix += @(
            @("wide-connected", "connected", 1920, 1080, "wide"),
            @("wide-twenty-tabs", "twenty_tabs", 1920, 1080, "wide"),
            @("wide-grid", "grid", 1920, 1080, "wide"),
            @("wide-editor", "editor", 1920, 1080, "wide"),
            @("wide-settings", "settings", 1920, 1080, "wide"),
            @("wide-import", "import", 1920, 1080, "wide")
        )
    }
    return ,$matrix
}

function New-SyntheticVisualReport {
    param([Parameter(Mandatory)][bool]$PlatformIsWindows)

    $visual = [ordered]@{}
    $paths = @()
    $pngInfo = @{}
    foreach ($entry in (Get-ExpectedMatrix -PlatformIsWindows $PlatformIsWindows)) {
        $id, $state, $width, $height, $layout = $entry
        $leaf = "production-p0-report-$id.png"
        $paths += $leaf
        $pngInfo[$leaf] = [pscustomobject]@{ width = $width; height = $height }
        $visual[$id] = [pscustomobject]@{
            checkpoint_id = $id
            state = $state
            layout = $layout
            facts = [pscustomobject]@{
                requested_width = $width
                requested_height = $height
                realized_width = $width
                realized_height = $height
                focus_or_selection_treatment = $true
                terminal_glyph_clipped_cells = 0
                terminal_min_line_separation = 2.0
            }
            png = [pscustomobject]@{
                width = $width
                height = $height
                non_empty = $true
                dark_regions_required = 4
                dark_regions_passed = 4
                focus_or_selection_thickness_px = 2
            }
            dpi = [pscustomobject]@{
                logical_width = $width
                logical_height = $height
                effective_scale = 1.0
                effective_dpi = 96.0
                icon_logical_size = 16
                icon_texture_width = 16
                icon_texture_height = 16
            }
            accessibility = [pscustomobject]@{
                unnamed_icon_controls = 0
                hidden_primary_actions = 0
                zero_size_panes = 0
                horizontal_clipping = $false
                background_insensitive = $true
                focus_contained = $true
                focus_restored = $true
            }
        }
    }
    return [pscustomobject]@{
        Report = [pscustomobject]@{
            visual = [pscustomobject]$visual
            png_paths = $paths
            requested_png_paths = @($paths)
        }
        PngInfo = $pngInfo
    }
}

function Copy-ContractValue {
    param([Parameter(Mandatory)]$Value)
    return $Value | ConvertTo-Json -Depth 20 | ConvertFrom-Json -Depth 20
}

function Copy-PngInfo {
    param([Parameter(Mandatory)][hashtable]$PngInfo)
    $copy = @{}
    foreach ($key in $PngInfo.Keys) {
        $copy[$key] = [pscustomobject]@{
            width = $PngInfo[$key].width
            height = $PngInfo[$key].height
        }
    }
    return $copy
}

function Assert-ContractPasses {
    param(
        [Parameter(Mandatory)][scriptblock]$Validator,
        [Parameter(Mandatory)]$Report,
        [Parameter(Mandatory)][hashtable]$PngInfo,
        [Parameter(Mandatory)][string]$Case
    )
    try {
        & $Validator -Report $Report -PngInfo $PngInfo
    }
    catch {
        throw "$Case should pass Assert-VisualContract but failed: $($_.Exception.Message)"
    }
}

function Assert-ContractFails {
    param(
        [Parameter(Mandatory)][scriptblock]$Validator,
        [Parameter(Mandatory)]$Report,
        [Parameter(Mandatory)][hashtable]$PngInfo,
        [Parameter(Mandatory)][string]$Case,
        [Parameter(Mandatory)][string]$ExpectedMessage
    )
    try {
        & $Validator -Report $Report -PngInfo $PngInfo
    }
    catch {
        if ($_.Exception.Message -notlike "*$ExpectedMessage*") {
            throw "$Case should fail with '$ExpectedMessage' but failed with: $($_.Exception.Message)"
        }
        return
    }
    throw "$Case should fail Assert-VisualContract but passed."
}

function Invoke-SyntheticContractTests {
    $failures = @()
    foreach ($platform in @(
            [pscustomobject]@{ Name = "NonWindows"; IsWindows = $false },
            [pscustomobject]@{ Name = "Windows"; IsWindows = $true }
        )) {
        $validator = Get-VisualContractValidator -PlatformIsWindows $platform.IsWindows
        $fixture = New-SyntheticVisualReport -PlatformIsWindows $platform.IsWindows
        if ($platform.IsWindows) {
            $aliasReport = Copy-ContractValue $fixture.Report
            $aliasPngInfo = Copy-PngInfo $fixture.PngInfo
            $standardLeaf = "production-p0-report-standard-connected.png"
            $decoyLeaf = "production-p0-report-standard-connected-decoy.png"
            $standardIndex = [array]::IndexOf([object[]]$aliasReport.png_paths, $standardLeaf)
            if ($standardIndex -lt 0) {
                throw "Windows alias fixture is missing the standard checkpoint PNG."
            }
            $aliasReport.png_paths[$standardIndex] = $decoyLeaf
            $aliasPngInfo.Remove($standardLeaf)
            $aliasPngInfo[$decoyLeaf] = [pscustomobject]@{ width = 1360; height = 860 }
            Assert-ContractFails $validator $aliasReport $aliasPngInfo `
                "Windows tail cannot alias standard checkpoint" `
                "P0 visual checkpoint PNG binding is ambiguous."
        }
        try {
            Assert-ContractPasses $validator $fixture.Report $fixture.PngInfo "$($platform.Name) valid matrix"
        }
        catch {
            $failures += $_.Exception.Message
            continue
        }

        $missing = Copy-ContractValue $fixture.Report
        $firstId = [string]@($missing.visual.PSObject.Properties)[0].Name
        $missing.visual.PSObject.Properties.Remove($firstId)
        Assert-ContractFails $validator $missing (Copy-PngInfo $fixture.PngInfo) `
            "$($platform.Name) missing checkpoint" `
            "P0 visual matrix evidence is incomplete."

        $duplicate = Copy-ContractValue $fixture.Report
        $duplicate.png_paths[1] = $duplicate.png_paths[0]
        Assert-ContractFails $validator $duplicate (Copy-PngInfo $fixture.PngInfo) `
            "$($platform.Name) duplicate PNG" `
            "P0 visual matrix PNG paths are incomplete or duplicated."

        $missingPngInfo = Copy-PngInfo $fixture.PngInfo
        $missingPngInfo.Remove([string]$fixture.Report.png_paths[0])
        Assert-ContractFails $validator (Copy-ContractValue $fixture.Report) $missingPngInfo `
            "$($platform.Name) missing PNG evidence" `
            "P0 visual checkpoint PNG is missing."

        $wrongDimensions = Copy-ContractValue $fixture.Report
        $wrongPngInfo = Copy-PngInfo $fixture.PngInfo
        $firstLeaf = [string]$wrongDimensions.png_paths[0]
        $wrongPngInfo[$firstLeaf].width++
        Assert-ContractFails $validator $wrongDimensions $wrongPngInfo `
            "$($platform.Name) wrong dimensions" `
            "P0 visual checkpoint dimensions are inconsistent."

        $wrongAccessibility = Copy-ContractValue $fixture.Report
        @($wrongAccessibility.visual.PSObject.Properties)[0].Value.accessibility.hidden_primary_actions = 1
        Assert-ContractFails $validator $wrongAccessibility (Copy-PngInfo $fixture.PngInfo) `
            "$($platform.Name) accessibility" `
            "P0 accessibility or contrast evidence failed."
        $negativeCases = if ($platform.IsWindows) { 6 } else { 5 }
        "VISUAL_CONTRACT_PLATFORM_PASS platform=$($platform.Name) checkpoints=26 negative_cases=$negativeCases"
    }
    if ($failures.Count -ne 0) {
        throw ($failures -join [Environment]::NewLine)
    }
    "VISUAL_CONTRACT_SYNTHETIC_PASS platforms=2 checkpoints=26 negative_cases=11"
}

function Read-PngContractInfo {
    param([Parameter(Mandatory)][string]$Path)

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $signature = [byte[]](137, 80, 78, 71, 13, 10, 26, 10)
    if ($bytes.Length -lt 24) {
        throw "PNG is too short: $Path"
    }
    for ($index = 0; $index -lt $signature.Length; $index++) {
        if ($bytes[$index] -ne $signature[$index]) {
            throw "PNG signature is invalid: $Path"
        }
    }
    if ($bytes[12] -ne 73 -or $bytes[13] -ne 72 -or $bytes[14] -ne 68 -or $bytes[15] -ne 82) {
        throw "PNG first chunk is not IHDR: $Path"
    }
    [uint32]$width = ([uint32]$bytes[16] * 16777216) +
        ([uint32]$bytes[17] * 65536) + ([uint32]$bytes[18] * 256) + $bytes[19]
    [uint32]$height = ([uint32]$bytes[20] * 16777216) +
        ([uint32]$bytes[21] * 65536) + ([uint32]$bytes[22] * 256) + $bytes[23]
    if ($width -eq 0 -or $height -eq 0) {
        throw "PNG IHDR dimensions are invalid: $Path"
    }
    $sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
    return [pscustomobject]@{ width = $width; height = $height; sha256 = $sha256 }
}

function Invoke-RetainedArtifactReplay {
    if ([string]::IsNullOrWhiteSpace($ReplayReport) -or
        [string]::IsNullOrWhiteSpace($ReplayRoot) -or
        [string]::IsNullOrWhiteSpace($ReplayPlatform)) {
        throw "ReplayReport, ReplayRoot, and ReplayPlatform are all required for replay."
    }
    $platformIsWindows = $ReplayPlatform -ceq "Windows"
    $report = Get-Content -LiteralPath $ReplayReport -Raw | ConvertFrom-Json -Depth 20
    $names = @($report.png_paths | ForEach-Object { [string]$_ })
    $requestedNames = @($report.requested_png_paths | ForEach-Object { [string]$_ })
    if ($names.Count -ne 26 -or @($names | Select-Object -Unique).Count -ne 26) {
        throw "Retained report must name 26 unique checkpoint PNGs."
    }
    if (@(Compare-Object -ReferenceObject $names -DifferenceObject $requestedNames).Count -ne 0) {
        throw "Retained requested and produced PNG names differ."
    }

    $pngInfo = @{}
    $records = @()
    foreach ($name in $names) {
        if ([System.IO.Path]::IsPathRooted($name) -or [System.IO.Path]::GetFileName($name) -cne $name) {
            throw "Retained report contains a non-leaf PNG name."
        }
        $path = Join-Path $ReplayRoot $name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Retained checkpoint PNG is missing: $name"
        }
        $info = Read-PngContractInfo -Path $path
        $pngInfo[$name] = $info
        $records += [pscustomobject]@{
            name = $name
            width = $info.width
            height = $info.height
            sha256 = $info.sha256
        }
    }
    $manifestLines = @($records | Sort-Object name | ForEach-Object {
            "$($_.name)|$($_.width)x$($_.height)|$($_.sha256)"
        })
    $manifestBytes = [Text.Encoding]::UTF8.GetBytes($manifestLines -join "`n")
    $manifestSha256 = [Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData($manifestBytes)
    ).ToLowerInvariant()
    foreach ($record in ($records | Sort-Object name)) {
        "VISUAL_REPLAY_FILE platform=$ReplayPlatform name=$($record.name) width=$($record.width) height=$($record.height) sha256=$($record.sha256)"
    }
    $dimensionSummary = @($records | Group-Object width, height | Sort-Object Name | ForEach-Object {
            "$($_.Name -replace ', ', 'x')=$($_.Count)"
        }) -join ","
    "VISUAL_REPLAY_FILES_OK platform=$ReplayPlatform checkpoints=26 dimensions=$dimensionSummary manifest_sha256=$manifestSha256"

    $validator = Get-VisualContractValidator -PlatformIsWindows $platformIsWindows
    Assert-ContractPasses $validator $report $pngInfo "$ReplayPlatform retained artifact"
    "VISUAL_REPLAY_PASS platform=$ReplayPlatform checkpoints=26 dimensions=$dimensionSummary manifest_sha256=$manifestSha256"
}

$replayRequested = -not [string]::IsNullOrWhiteSpace($ReplayReport) -or
    -not [string]::IsNullOrWhiteSpace($ReplayRoot) -or
    -not [string]::IsNullOrWhiteSpace($ReplayPlatform)
if ($replayRequested) {
    Invoke-RetainedArtifactReplay
}
else {
    Invoke-SyntheticContractTests
}
