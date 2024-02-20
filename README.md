# NoBlockInput

NoBlockInput hooks the call to [BlockInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-blockinput) from remote desktop software, preventing the application from being able to effectively call it and block user input.
As of 0.5.0, it also hooks [SendInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) to block any input from the remote side.
Note that it is outside the scope of this tool to unhide a screen that has been hidden by remote desktop software, although it is possible to do so.

### Important known issue
There are some problems with newer functionality when used with certain remote access software. UltraViewer and SupRemo are known to be incompatible with the current release. You can track the progress of fixes in [this issue](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/issues/9).

### Documentation
See the wiki for [a guide to getting started](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Getting-Started-and-Usage) and [configuration](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Configuration).


### Remote desktop software support
The highest priority is to effectively support modifying the behavior of ConnectWise ScreenConnect with modifying behavior of other remote desktop software being the second priority.
Note that this tool does not aim to have support for all remote desktop software, but this should support hooking and blocking calls from *any* process.
