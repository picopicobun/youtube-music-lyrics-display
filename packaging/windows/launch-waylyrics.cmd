@echo off
setlocal
set "WAYLYRICS_THEME_PRESETS_DIR=%~dp0themes"
set "XDG_DATA_DIRS=%~dp0share"
set "GSETTINGS_SCHEMA_DIR=%~dp0share\glib-2.0\schemas"
set "GDK_PIXBUF_MODULE_FILE=%~dp0lib\gdk-pixbuf-2.0\2.10.0\loaders.cache"
set "PATH=%~dp0bin;%~dp0;%PATH%"
start "Waylyrics" "%~dp0waylyrics.exe"
endlocal
