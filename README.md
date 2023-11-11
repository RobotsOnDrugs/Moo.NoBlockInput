# NoBlockInput

NoBlockInput hooks the call to [BlockInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-blockinput) from remote desktop software, preventing the application from being able to effectively call it and block user input.

**Note:** in order to hook an API call in an application and pause its execution to hook before it can call the API, administrator privileges are required. Also note that it is outside the scope of this tool to unhide a screen that has been hidden by remote desktop software, although it is possible to do so.

### Usage
NoBlockInput is most effective when run before any remote desktop software.
Once launched, it will sit in the background and wait for such software to run, hooking it as soon as it receives the process creation event from Windows.
It can be run while the remote desktop software is running and hook it, but cannot truly unblock input once the software has blocked it.
However, killing such software remotely and letting it run again is an effective workaround as the tool will detect and hook it immediately.

### Remote desktop software support
Note that it is this list is not exhaustive of all remote desktop software and that this tool does not aim to have support for everything. If specific software becomes used by scammers enough to be notable, it will be placed on this list and eventually supported as applicable.

Outright malicious remote access tools (RATs) are not planned for investigation at this time but could be supported in the future if needed.
| Software | Status | Remarks |
| --- | :---: | --- |
| [AnyDesk](https://anydesk.com/) | :x: | Input blocking can be turned off by normal means |
| [AweSun/AweRay Remote](https://sun.aweray.com/) | :x: | Planned for investigation |
| [Chrome Remote Desktop](https://remotedesktop.google.com) | :x: | Planned for investigation |
| [ConnectWise ScreenConnect](https://screenconnect.connectwise.com/) | ✔️ | Fullly supported, 64-bit |
| [GoToMyPC](https://get.gotomypc.com/) | :x: | Planned for investigation (low priority) |
| [LogMeInRescue](https://www.logmeinrescue.com/)/[LogMeIn123](https://secure.logmeinrescue.com/customer/code.aspx) | :x: | Planned for investigation |
| [Remote Utilities](https://www.remoteutilities.com/) | ✔️ | Fully supported, 32-bit |
| [RustDesk](https://rustdesk.com/) | :x: | Planned for investigation |
| [Splashtop](https://www.splashtop.com/) | :x: | Planned for investigation |
| [SupRemo](https://www.supremocontrol.com/) | :x: | Planned for investigation |
| [TeamViewer](https://www.teamviewer.com/) | :x: | Planned for investigation (low priority) |
| [UltraViewer](https://www.ultraviewer.net/) | N/A | Doesn't seem to support blocking input |
