# Annoying Process Killer
Simple utility to watch for and kill processes that start up

I made this because `CompatTelRunner.exe` wouldn't leave me alone, regardless of me following all the steps online to disable it.

Now it will never bother me again!

## Flags
`--hide` or `-h` will hide the console (for example if you want to autostart with Windows).

## Configuration
Just add any other processes you want to watch for and kill to the `config.json` file, then restart the program. You can also adjust the polling speed (in seconds). This file will be auto generate the first time you run the program.

## How it works
It uses the Windows API to first grant special permissions to the program so it can kill privileged processes, then secondly kills them when they start running.

## Notes
This will ask for admin, because it requires access to the `SE_DEBUG_NAME` privilege in order to kill SYSTEM processes.
