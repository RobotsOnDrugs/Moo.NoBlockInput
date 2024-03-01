# Requires pwsh (7.0+) and won't work on PowerShell 5 (for the conditional syntax; could be made compatible if necessary)
# Also requires cargo for obvious reasons
# Uses i686-pc-windows-msvc for 32-bit and x86_64-pc-windows-msvc for 64-bit
# You could probably use the mingw variants or whatever instead if you really needed to, but I only test msvc builds

# BTW, there's a bug where it will exit with no output and produce nothing if a previous build failed
# I'll fix it later, but for now, you can just tell it to clean and it'll work

<#
  .DESCRIPTION
  Builds the NoBlockInput injector and its associated DLL to inject for both x86 and x64. Final binaries will be located in .\build\release.
  .PARAMETER BuildDebug
  Optional. Specifies whether to produce a debug build, located in .\build\debug. If omitted, a release build is generated.
  .PARAMETER Clean
  Optional. Specifies whether to fully clean (run `cargo clean`) before building. If omitted, no cleaning is performed.
  .PARAMETER FileDescription <Description>
  Optional. Specifies the value of FileDescription in the binary's manifest, which is what is displayed in Task Manager
 #>
 #>
#>

param([Switch]$BuildDebug, [Switch]$Clean, [Parameter()][string]$FileDescription)
$ErrorActionPreference = 'Stop'
$injector_version = (cargo.exe read-manifest --manifest-path .\injector\Cargo.toml | ConvertFrom-Json).version
$hook_version = (cargo.exe read-manifest --manifest-path .\hook\Cargo.toml | ConvertFrom-Json).version
$common_version = (cargo.exe read-manifest --manifest-path .\common\Cargo.toml | ConvertFrom-Json).version
if ($injector_version -ne $hook_version)
{
    throw 'Injector version "${injector_version}", hook version ${hook_version}, and common version ${common_version} did not match. Fix the manifests.'
}

$release_type = ($BuildDebug ? "debug" : "release" )
$release_switch = ($BuildDebug ? "" : "release" )

$output_dir = ".\build\$release_type"
Remove-Item -Recurse -Force -Path $output_dir -ErrorAction SilentlyContinue | Out-Null
New-Item -Type Directory $output_dir | Out-Null

$env:BINARY_FILE_DESCRIPTION=$FileDescription

if ($Clean) { cargo.exe clean }
else { cargo.exe clean -r -p noblock_input_hook; cargo clean -r -p noblock_input_hook_injector; }
if ($LASTEXITCODE -eq 0) { cargo.exe build --target=i686-pc-windows-msvc --$release_switch }
if ($LASTEXITCODE -eq 0) { cargo.exe build --target=x86_64-pc-windows-msvc --$release_switch }
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$x86_path = ".\target\i686-pc-windows-msvc\$release_type"
$x64_path = ".\target\x86_64-pc-windows-msvc\$release_type"
Copy-Item $x86_path\noblock_input_hook.dll $output_dir\noblock_input32.dll
Copy-Item $x86_path\noblock_input_hook_injector.exe $output_dir\noblock_input32.exe
Copy-Item $x64_path\noblock_input_hook.dll $output_dir\noblock_input.dll
Copy-Item $x64_path\noblock_input_hook_injector.exe $output_dir\noblock_input.exe
Copy-Item .\injector\configuration\injector.toml $output_dir\injector.toml
Copy-Item .\injector\configuration\x86.reg $output_dir\x86.reg
Copy-Item .\injector\configuration\x64.reg $output_dir\x64.reg