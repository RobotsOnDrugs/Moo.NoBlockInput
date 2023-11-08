# NoBlockInput

This tool hooks the call to [BlockInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-blockinput) from remote access software, preventing the application from being able to effectively call it and block user input. **Note:** in order to hook an API call in an application and pause its execution to hook before it can call the API, administrator privileges are required.

Currently, it can only hook ScreenConnect client, but support for other software such as UltraViewer is planned.

### Usage
This tool is most effective when run before any remote access software.
Once launched, it will sit in the background and wait for such software to run, hooking it as soon as it receives the process creation event from Windows.
It can be run while the remote access software is running and hook it, but cannot truly unblock input once the software has blocked it.
However, killing such software remotely and letting it run again is an effective workaround as the tool will detect and hook it immediately.

It is recommended to run this tool as a scheduled task with highest privileges.