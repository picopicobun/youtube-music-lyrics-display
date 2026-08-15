$Env:WAYLYRICS_THEME_PRESETS_DIR = "..\share\waylyrics\themes"
$gtkRoot = $Env:GTK_ROOT
if (-not $gtkRoot) {
    throw "Set GTK_ROOT to the GTK installation directory before building on Windows."
}
$Env:GETTEXT_DIR = $gtkRoot
$Env:PKG_CONFIG_PATH = Join-Path $gtkRoot "lib\pkgconfig"
$Env:Path += ";$(Join-Path $gtkRoot 'bin')"

cargo build -j4 --release --no-default-features -F tray-icon -F i18n -F import-lyric -F export-lyric
