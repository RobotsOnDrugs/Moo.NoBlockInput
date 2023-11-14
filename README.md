# NoBlockInput

NoBlockInput hooks the call to [BlockInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-blockinput) from remote desktop software, preventing the application from being able to effectively call it and block user input.

**Note:** in order to hook an API call in an application and pause its execution to hook before it can call the API, administrator privileges are required. Hooking SupRemo requires SYSTEM-level privileges, so a tool such as [PsExec](https://learn.microsoft.com/en-us/sysinternals/downloads/psexec) with the `-s` switch is required. Also note that it is outside the scope of this tool to unhide a screen that has been hidden by remote desktop software, although it is possible to do so.

**Additional important note:** Windows Defender does not like this behavior and flags it as Win32/Wacapew.C!ml with "potentially unwanted behavior", which is apparently [a common false positive made by their AI](https://answers.microsoft.com/en-us/windows/forum/all/wacatac-false-positive-outbreak/0d92ef05-50db-4d12-92f4-fcfe8f0b966c) (note the "ml" suffix for "machine learning"). It even [flags things made with PyInstaller](https://github.com/pyinstaller/pyinstaller/issues/5668) (lol). McAfee does not (yet?) flag this program and is more fun to use with scammers on the VM anyway. Evading AV is not a priority for this project, so you'll have to make an exclusion for whatever folder you place this in if Defender is giving you trouble.

### Usage
NoBlockInput is most effective when run before any remote desktop software.
Once launched, it will sit in the background and wait for such software to run, hooking it as soon as it receives the process creation event from Windows.
It can be run while the remote desktop software is running and hook it, but cannot truly unblock input once the software has blocked it.
However, killing such software remotely and letting it run again is an effective workaround as the tool will detect and hook it immediately.

### Remote desktop software support
Note that this list is not exhaustive of all remote desktop software and that this tool does not aim to have support for everything. If specific software becomes used by scammers enough to be notable, it will be placed on this list and eventually supported as applicable.

Outright malicious remote access tools (RATs) are not planned for investigation at this time but could be supported in the future if needed.
| Software | Status | Remarks |
| --- | :---: | --- |
| [Ammyy Admin](https://www.ammyy.com/) | :x: | Planned for investigation |
| [AnyDesk](https://anydesk.com/) | :x: | Planned for investigation (low priority) |
| [AnyViewer](https://www.anyviewer.com/) | ➖ | Planned for investigation (low priority); doesn't seem to support blocking input in the free version |
| [AweSun/AweRay Remote](https://sun.aweray.com/) | ➖ | Can blank screen, but doesn't seem to support blocking input |
| [Chrome Remote Desktop](https://remotedesktop.google.com) | :x: | Investigation currently blocked (requires Google login on both sides) (low priority) |
| [ConnectWise ScreenConnect](https://screenconnect.connectwise.com/) | ✔️ | Fully supported, 64-bit |
| [GoToMyPC](https://get.gotomypc.com/) | ✖️ | Only some uncommon and expensive variations of GoToWhatever can block input, so this is low priority |
| [LogMeInRescue](https://www.logmeinrescue.com/)/[LogMeIn123](https://secure.logmeinrescue.com/customer/code.aspx) | ✖️ | The standard version doesn't support blocking input, and the pro version which does is expensive and seems to be uncommon, so this is low priority |
| [Quick Assist](https://apps.microsoft.com/detail/quick-assist/9P7BP5VNWKX5) | :x: | Planned for investigation (low priority) |
| [Remote Utilities](https://www.remoteutilities.com/) | ✔️ | Fully supported, 32-bit |
| [RustDesk](https://rustdesk.com/) | :x: | Planned for investigation |
| [Splashtop](https://www.splashtop.com/) | :x: | Planned for investigation |
| [SupRemo](https://www.supremocontrol.com/) | ✔️ | Fully supported, 32-bit; requires SYSTEM-level privileges |
| [TeamViewer](https://www.teamviewer.com/) | :x: | Planned for investigation (low priority) |
| [UltraViewer](https://www.ultraviewer.net/) | ➖ | Doesn't seem to support blocking input |
