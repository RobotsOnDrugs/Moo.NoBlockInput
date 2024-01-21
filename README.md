# NoBlockInput

NoBlockInput hooks the call to [BlockInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-blockinput) from remote desktop software, preventing the application from being able to effectively call it and block user input.
As of 0.5.0, it also hooks [SendInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) to block any input from the remote side.
Note that it is outside the scope of this tool to unhide a screen that has been hidden by remote desktop software, although it is possible to do so.

**Note:** in order to hook an API call in an application and pause its execution to hook before it can call the API, administrator privileges are required.

**Additional important note:** Windows Defender's AI heuristics do not like this behavior and flag it as Win32/Wacapew.C!ml with "potentially unwanted behavior", which is apparently [a common false positive made by their AI](https://answers.microsoft.com/en-us/windows/forum/all/wacatac-false-positive-outbreak/0d92ef05-50db-4d12-92f4-fcfe8f0b966c) (note the "ml" suffix for "machine learning").
It even [flags things made with PyInstaller](https://github.com/pyinstaller/pyinstaller/issues/5668) (lol).
Evading AV is not a priority for this project, so an exclusion for whatever folder it is in must be made if AV flags it.

### Usage
The injector.toml file specifies which processes to hook.
Note that there is a 32-bit version and a 64-bit version. The bitness must match the target processes. For example, ConnectWise ScreenConnect is 64-bit, so the 64-bit injector must be used.

NoBlockInput is most effective when run before any remote desktop software, however it will hook existing processes.\
Once launched, it will also sit in the background and wait for target processes to run, hooking them as soon as it receives the image load event from Windows.\
It can be run while the remote desktop software is running and hook it, but cannot unblock input once the software has blocked it.\
However, killing such software remotely and letting it run again is an effective workaround as the tool will detect and hook it immediately.

### Remote desktop software support
The highest priority is to effectively support modifying the behavior of ConnectWise ScreenConnect with modifying behavior of other remote desktop software being the second priority.
Note that this tool does not aim to have support for all remote desktop software, but this should support hooking and blocking calls from *any* process, at least in theory.