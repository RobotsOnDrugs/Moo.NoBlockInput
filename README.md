# NoBlockInput

NoBlockInput hooks the call to [BlockInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-blockinput) from remote desktop software (RDS) to block the application from being able to block physical mouse and keyboard input as well as [SendInput()](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) to block any input from the remote side.

## Features
- RDS can be rendered unable to block physical keyboard and mouse input on the remote side. This can be toggled on/off at any time via the registry.
- RDS can be rendered unable to send any keyboard and mouse input. This can be toggled on/off at any time.
- Both 32-bit and 64 bit processes can be hooked.[^1]
- BlockInput() and SendInput() can be blocked for any arbitrary process, though only RDS is tested.
- Processes in the configuration are monitored and will be hooked on start and unhooked on exit. All such processes will be unhooked when the program is closed via Ctrl-C.[^2]
- The build script supports changing the binary manifest information to show any arbitrary name in the file properties and the name the name that is shown in Task Manager.[^3]
- The program logs to both the terminal[^4] and optionally to file at a configurable location.

### Planned features
- Migration from TOML configuration to registry configuration.[^5] ([Issue #8](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/issues/8)).
- Configuration of random allowance of remote input to annoy the scammer.
- Removal or even configurable opacity of any privacy screen used by RDS. ([Issue #11](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/issues/11)).

Please feel free to comment on any issue and add a thumbs up emoji to the initial post if you'd like to have it prioritized. Enhancement requests and bug reports are also very welcome.


## Documentation
See the wiki for [a guide to getting started](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Getting-Started-and-Usage) and [configuration](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Configuration).


## Known issues
This tool aims to have support for most RDS, but there are some that either have compatiblity issues or are untested. See [issue #9](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/issues/9) and [the compatibility wiki page](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Current-compatibility-with-remote-access-software) for progress of testing and compatibility fixes.

## Other
"Moo" in the name comes from an inside joke among [Scam Fight Club](https://www.scamfightclub.com/) members based on the fact that we like to get scammers to moo.\
If you're a YouTuber and/or Twitch streamer who uses this tool, [send me a DM on Discord](https://discord.com/users/487390628473208833) with a link to a video or stream VOD. I would love to see it in action.

<br>

Notes
[^1]: There are separate program binaries and separate configuration entries for each bitness.
[^2]: Processes cannot be unhooked if the program is terminated unexpectedly and must be restarted in order to run unhooked. See  [the getting started guide](https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Getting-Started-and-Usage#41-usage-of-windows-kill) for important information on release builds.
[^3]: If the DLL names do not match their respective injector names, their names must be specified in the configuration.
[^4]: Only debug builds will write logging information to the terminal in order to prevent showing a console window in release builds.
[^5]: Support for TOML configuration will be removed in the future. It will still be possible to migrate.
