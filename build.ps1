$Env:WAYLYRICS_THEME_PRESETS_DIR = "..\share\waylyrics\themes"
$gtkRoot = $Env:GTK_ROOT
if (-not $gtkRoot) {
    throw "Set GTK_ROOT to the GTK installation directory before building on Windows."
}
$Env:GETTEXT_DIR = $gtkRoot
$Env:PKG_CONFIG_PATH = Join-Path $gtkRoot "lib\pkgconfig"
$Env:Path += ";$(Join-Path $gtkRoot 'bin')"

# Keep build-machine paths out of the portable executable and its native
# dependencies. Besides protecting privacy, this makes release builds more
# reproducible across maintainer machines.
$remapFrom = $Env:USERPROFILE
$remapFromForward = $remapFrom.Replace("\", "/")
$Env:RUSTFLAGS = "--remap-path-prefix=$remapFrom=C:\build -C strip=symbols"
$Env:CFLAGS = "-ffile-prefix-map=$remapFromForward=C:/build"
$Env:CXXFLAGS = $Env:CFLAGS
$Env:CMAKE_C_FLAGS = $Env:CFLAGS
$Env:CMAKE_CXX_FLAGS = $Env:CXXFLAGS

$rustup = Join-Path $Env:USERPROFILE ".cargo\bin\rustup.exe"
if (-not (Test-Path -LiteralPath $rustup)) {
    throw "rustup.exe was not found. Install Rust before building on Windows."
}

& $rustup run stable-x86_64-pc-windows-gnu cargo build -j4 --release --no-default-features -F tray-icon -F i18n -F import-lyric -F export-lyric
if ($LASTEXITCODE -ne 0) {
    throw "Failed to build Waylyrics for Windows GNU."
}
