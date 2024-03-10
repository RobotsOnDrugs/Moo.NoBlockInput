# NoBlockInput

NoBlockInput hooks the call to [BlockInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-blockinput) from remote desktop software to block the application from being able to block physical mouse and keyboard input as well as [SendInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) to block any input from the remote side.\
Note that it is currently outside the scope of this tool to unhide a screen that has been hidden by remote desktop software, although it may be possible to do so.

### Documentation
See the wiki for [a guide to getting started](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Getting-Started-and-Usage) and [configuration](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Configuration).

### Known issues
This tool aims to have support for most remote desktop software, but there are some that either have compatiblity issues or are untested. See [issue #9](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/issues/9) and [the compatibility wiki page](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Current-compatibility-with-remote-access-software) for progress of testing and compatibility fixes.
