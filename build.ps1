# This script requires PowerShell 7.0+ and won't work on PowerShell 5 because of the conditional syntax, but could be made compatible if necessary
# Uses i686-pc-windows-msvc for 32-bit and x86_64-pc-windows-msvc for 64-bit
# MinGW builds will likely work, but only msvc builds are tested

# Note: there's a bug where it will exit with no output and produce nothing if a previous build failed
# This will be fixed eventually, but for now, use the -Clean parameter to reset it

<#
  .DESCRIPTION
  Builds the NoBlockInput injector and its associated DLL to inject for both x86 and x64. Final binaries will be copied to .\build.
  .PARAMETER Clean
  Optional. Specifies whether to fully clean (run `cargo clean`) before building. If omitted, no cleaning is performed.
  .PARAMETER Build <Type>
  Optional. Specifies which release type(s) to build. Valid values are 'DebugOnly', 'ReleaseOnly', and 'All'. If 'All' is chosen or this parameter is omitted, both debug and release builds will be built.
  .PARAMETER FileDescription <Description>
  Optional. Specifies the value of FileDescription in the binary's manifest, which is what will displayed in Task Manager. Specifying this with something innocuous this is recommended in case a very suspicious scammer looks at Task Manager.
  .PARAMETER AdditionalOutputDirs
  Optional. An array of additional directories to copy binaries to. Must be absolute paths.
#>

param(
	[Switch]$Clean,
	[Parameter()][string][ValidateSet('DebugOnly','ReleaseOnly','All')]$Build = 'All',
	[Parameter()][string]$FileDescription = 'Moo.NoBlockInput',
	[Parameter()][System.Collections.Generic.HashSet[String]]$AdditionalOutputDirs
)
$ErrorActionPreference = 'Stop'

$injector_version = (cargo.exe read-manifest --manifest-path .\injector\Cargo.toml | ConvertFrom-Json).version
$hook_version = (cargo.exe read-manifest --manifest-path .\hook\Cargo.toml | ConvertFrom-Json).version
$common_version = (cargo.exe read-manifest --manifest-path .\common\Cargo.toml | ConvertFrom-Json).version
$versions = @($injector_version, $hook_version, $common_version)
$unique = (ForEach-Object {$versions} | Get-Unique)
if ($unique.count -ne 1)
{
	Write-Host "Injector version: "$versions[0]
	Write-Host "Hook version: " $versions[1]
	Write-Host "Common version: " $versions[2]
	throw 'Versions did not match. Fix the manifests.'
}

$env:BINARY_FILE_DESCRIPTION=$FileDescription

if ($Clean) { cargo.exe clean }
else
{
	cargo.exe clean -p noblock_input_hook;
	cargo.exe clean -p noblock_input_hook_injector;
	cargo.exe clean -r -p noblock_input_hook;
	cargo.exe clean -r -p noblock_input_hook_injector;
}

$targets = @('i686-pc-windows-msvc', 'x86_64-pc-windows-msvc')

if ($LASTEXITCODE -eq 0)
{
	Out-Null $(Remove-Item -Recurse -Force -Path '.\build' -ErrorAction SilentlyContinue)
	$build_debug, $build_release = switch ($Build)
	{
		'DebugOnly' { @($true, $false) }
		'ReleaseOnly' { @($false, $true) }
		'All' { @($true, $true) }
	}

	foreach ($target in $targets)
	{
		if ($target -eq 'i686-pc-windows-msvc') { $suffix = '32' } else { $suffix = $null }
		$subdirs = @()
		if ($build_debug) { cargo.exe +nightly build --target="${target}"; $subdirs += 'debug' }
		if ($build_release) { cargo.exe +nightly build --target="${target}" --release; $subdirs += 'release' }
		foreach ($subdir in $subdirs)
		{
			$orig_path = ".\target\${target}\$subdir"
			$new_path = ".\build\${subdir}"
			Out-Null $(New-Item -Force -Type Directory $new_path)
			Get-ChildItem $orig_path | Where-Object { $_.Extension -eq '.exe' -or $_.Extension -eq '.dll' } | ForEach-Object { Copy-Item -Path "$_" -Destination "${new_path}\$($FileDescription)${suffix}$($_.Extension)" }
			Copy-Item .\injector\configuration\configuration_readme.md $new_path\configuration_readme.md
			Copy-Item .\injector\configuration\x86.reg $new_path\x86.reg
			Copy-Item .\injector\configuration\x64.reg $new_path\x64.reg
		}
	}
}
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

foreach ($additional_output_dir in $AdditionalOutputDirs)
{
	if (![System.IO.Path]::IsPathRooted($additional_output_dir))
	{
		Write-Error "'$($additional_output_dir)' is not an absolute path!"
	}
	if (!(Test-Path $additional_output_dir))
	{
		Out-Null $(New-Item -Type Directory $additional_output_dir)
	}
	$new_build_dir = Join-Path -Path $additional_output_dir -ChildPath 'build'
	Out-Null $(Remove-Item -Recurse -Path $new_build_dir -ErrorAction SilentlyContinue)
	Copy-Item -Recurse '.\build' $additional_output_dir
}