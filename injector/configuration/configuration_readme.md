All configuration is stored in the registry, and registry files containing the default values are provided therein.\
See https://github.com/RobotsOnDrugs/Moo.NoBlockInput/wiki/Configuration for a full explanation of the configuration.


## Options

- **Hooks - `BlockInputHookEnabled` and `SendInputHookEnabled`**\
  *By default, these are set to 1, which enables the hooks. You can disable one or both by setting the respctive value to 0.*

- **Hook DLL name - `HookDllName`**\
  *This is the name of the DLL (with the .dll extension) to use. Use this only if you really need to have the DLL with a different name. This option will probably be removed in the future.*

- **Log directory - `LogDirectory`**\
  *The full path of the directory where you would like log files to be stored. If you leave this blank, no log files will be generated, but logging messages will still be displayed in the console.*

- **Processes**\
  *The processes are the names of the exact processes that call BlockInput().
  The defaults are known to be culprits, but you can add your own for other remote desktop software which may or may not be compatible. Don't get the bitness wrong.*

- **Trace name**\
  *The trace name can be anything you like that is not already in use by the system.
  You can use the logman utility which ships with Windows to see a list of running sessions (`logman query -ets`).
  Alternatively, use tracelog which ships with the Windows Driver Kit, Visual Studio, and the Windows SDK for more a more detailed view (`tracelog -l`).*