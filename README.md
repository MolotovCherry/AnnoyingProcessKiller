# AnnoyingProcessKiller
Simple utility to watch for and kill processes that start up

I made this because `CompatTelRunner.exe` wouldn't leave me alone, regardless of me following all the steps online to disable it.

Now it will never bother me again!

## Flags
`--hide` or `-h` will hide the console (for example if you want to autostart with Windows).

## Configuration
Just add any other processes you want to watch for and kill to the config.json file, then restart the program. You can also adjust the polling speed (in seconds).

## How it works
It uses the Windows API to grant special permissions to the program so it can kill privileged processes.

## Setup
Run as admin if you're trying to kill system processes (like `CompatTelRunner.exe`).
