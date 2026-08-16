# Windows 小白安装教程

这份教程适用于 Windows 10 和 Windows 11。整个过程不需要 Visual Studio，也不需要安装 Rust、GTK 或其他开发工具。

## 第一步：下载正确的文件

点击下面这个按钮下载 Windows 便携版：

## [⬇️ 直接下载 Waylyrics Windows 便携版](https://github.com/picopicobun/youtube-music-lyrics-display/releases/latest/download/DOWNLOAD-ME-Waylyrics-Windows-Portable.zip)

如果上面的直接下载按钮暂时打不开，请进入[最新版本页面](https://github.com/picopicobun/youtube-music-lyrics-display/releases/latest)，找到页面底部的 **Assets**，只下载 `DOWNLOAD-ME-Waylyrics-Windows-Portable.zip`。其他 `Source code` 文件是给开发者使用的源码，不需要下载。

> [!WARNING]
> 不要下载 GitHub 自动生成的 `Source code (zip)` 或 `Source code (tar.gz)`。那两个是给开发者编译用的源码，普通用户无法直接运行。

## 第二步：完整解压 ZIP

1. 打开 Windows 的“下载”文件夹。
2. 找到刚下载的 `DOWNLOAD-ME-Waylyrics-Windows-Portable.zip`。
3. 右键这个 ZIP，选择 **全部解压**。
4. 选择一个容易找到的位置，例如桌面，然后点击 **提取**。
5. 等待解压完成，再打开解压后出现的文件夹。

> [!IMPORTANT]
> 不要直接在 ZIP 的预览窗口里运行程序。Waylyrics 需要同时读取旁边的 `bin`、`lib`、`share` 和 `themes` 文件夹，所以必须先完整解压。

## 第三步：启动 Waylyrics

在解压后的文件夹根目录里找到：

`waylyrics.exe`

双击它即可启动。这个 EXE 是一个很小的启动器，会自动找到 `bin` 文件夹中的 GTK 运行库，再打开真正的 Waylyrics。正常情况下不会出现黑色代码窗口，桌面上会出现歌词窗口，任务栏右下角的系统托盘里也会出现 Waylyrics 图标。

也可以双击 `launch-waylyrics.vbs`，效果相同。请不要把任何一个启动文件单独移动出去；整个文件夹需要保持在一起。

## 第四步：处理 Windows 安全提示

这个个人开源版本没有购买商业代码签名证书，所以第一次运行时 Windows 可能显示“Windows 已保护你的电脑”或“未知发布者”。

如果文件是从本仓库的 Releases 下载的：

1. 点击提示窗口里的 **更多信息**。
2. 确认应用名称是 `waylyrics.exe`。
3. 点击 **仍要运行**。

如果 ZIP 的右键菜单里有“属性”，也可以先打开属性，在底部勾选 **解除锁定**，点击“确定”后再解压。

## 第五步：让歌词显示出来

1. 打开 YouTube Music、Spotify 或其他能出现在 Windows 媒体控制里的播放器。
2. 播放一首歌曲，不要只停留在歌曲页面。
3. 等待约 2～5 秒，Waylyrics 会自动识别歌曲并搜索歌词。
4. 当前歌词会以较大的发光文字显示，下一句歌词会在下方以较小、较暗的文字显示。

本版本默认同时使用 **网易云音乐** 和 **LRCLib** 搜索歌词。QQ 音乐词源也保留在程序中，但需要额外运行 QQMusicApi 服务，因此没有默认开启。

## 第六步：调整外观和鼠标穿透

点击歌词窗口右上角的“三条横线”按钮打开菜单：

- `Toggle Passthrough`：开启或关闭鼠标穿透。建议先把窗口移动到合适位置，再开启穿透。
- `Appearance`：实时调整当前歌词和下一句歌词的字号。
- `Glow`：调整发光颜色。
- `Colors`：调整当前歌词和下一句歌词的颜色。
- `Refetch lyric`：重新搜索当前歌曲的歌词。
- `Select Player`：手动选择要连接的播放器。

如果开启鼠标穿透后无法点击窗口，可以使用系统托盘里的 Waylyrics 菜单关闭鼠标穿透。

## 可选：创建桌面快捷方式

1. 右键根目录的 `waylyrics.exe`。
2. Windows 11 用户先点击 **显示更多选项**。
3. 选择 **发送到 → 桌面快捷方式**。

以后直接双击桌面快捷方式即可。不要删除原来的便携版文件夹。

## 常见问题

### 窗口出现了，但没有歌词

按顺序尝试：

1. 确认播放器正在播放，而不是停在页面上。
2. 等待 5 秒，或切换到另一首歌再切回来。
3. 打开 Waylyrics 菜单，点击 `Refetch lyric`。
4. 打开 `Select Player`，手动选择当前播放器。
5. 完全退出 Waylyrics 后，再次双击根目录的 `waylyrics.exe`。

从 `v0.4.0-pico.2` 开始，空歌词缓存会被自动忽略并重新搜索，不会再一直卡在空白状态。

### 双击后什么都没发生

查看任务栏右下角的系统托盘，Waylyrics 可能已经在后台运行。也可以打开任务管理器，结束已有的 `waylyrics-app.exe`，再重新双击根目录的 `waylyrics.exe`。

### 小狐狸图标出现一秒就消失

请使用 `v0.4.0-pico.4` 或更新版本重新启动。新版启动器会在便携版根目录生成 `waylyrics-startup.log`；如果主程序在 5 秒内退出，还会弹出退出状态和日志位置。

请把 `waylyrics-startup.log` 的完整内容发给维护者。同时打开 **Windows 安全中心 → 病毒和威胁防护 → 保护历史记录**，检查 `waylyrics-app.exe` 是否被安全软件拦截或隔离。

### 提示缺少 `libgio-2.0-0.dll` 或其他 DLL

请确认你下载的是 `v0.4.0-pico.3` 或更新版本，并且已经完整解压。从这个版本开始，根目录的 `waylyrics.exe` 是专用启动器，会自动加载 `bin` 文件夹里的 DLL。旧版本若直接双击主程序，可能出现这个错误。

### 歌词窗口挡住鼠标点击

打开右上角菜单，开启 `Toggle Passthrough`。开启前请先调整好窗口位置。

### 杀毒软件提示风险

这是没有商业签名的个人编译版本，部分安全软件可能会对新发布、下载量较少的 EXE 提示风险。请只从本仓库的 Releases 下载，并可在 Release 页面使用 SHA-256 校验值核对文件。

## 更新版本

设置、歌词缓存和外观配置保存在 Windows 用户的 AppData 中，不在便携版文件夹里。更新时可以下载新的 ZIP，解压到一个新文件夹并启动，原来的设置通常会自动沿用。

## 卸载

1. 从系统托盘完全退出 Waylyrics。
2. 删除解压出来的便携版文件夹和桌面快捷方式。

这样即可卸载程序。若还想同时清除全部设置和歌词缓存，可另外删除：

- `%APPDATA%\poly000\waylyrics`
- `%LOCALAPPDATA%\poly000\waylyrics`

删除这两个目录后，字号、颜色、主题和缓存都无法恢复；只想卸载程序时不需要删除它们。

## 项目来源

此 Windows 定制版由 **picopicobun** 维护，基于开源项目 [waylyrics/waylyrics](https://github.com/waylyrics/waylyrics) v0.4.0。原项目版权、MIT 许可证和贡献者信息均予以保留。
