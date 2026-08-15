Set shell = CreateObject("WScript.Shell")
installDir = Left(WScript.ScriptFullName, InStrRev(WScript.ScriptFullName, "\"))

shell.Environment("Process")("WAYLYRICS_THEME_PRESETS_DIR") = installDir & "themes"
shell.Environment("Process")("XDG_DATA_DIRS") = installDir & "share"
shell.Environment("Process")("GSETTINGS_SCHEMA_DIR") = installDir & "share\glib-2.0\schemas"
shell.Environment("Process")("GDK_PIXBUF_MODULE_FILE") = installDir & "lib\gdk-pixbuf-2.0\2.10.0\loaders.cache"
shell.Environment("Process")("Path") = installDir & "bin;" & installDir & shell.Environment("Process")("Path")

shell.Run Chr(34) & installDir & "waylyrics.exe" & Chr(34), 1, False
