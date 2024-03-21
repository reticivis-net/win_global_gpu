# ![Win Global GPU](assets/banner.png)

# Win Global GPU

Created by [Melody](https://reticivis.net/)

Written in [Rust](https://www.rust-lang.org/)

## What is Win Global GPU?

Win Global GPU lets you globally use either your dedicated or integrated GPU. You can manually specify which one or have
it automatically set when you plug in or unplug your laptop.

## How to use it?

Download the exe file in the [releases tab](https://github.com/reticivis-net/uwd2/releases)

Run it in your favorite terminal: `win_global_gpu.exe` or `win_global_gpu.exe SUBCOMMAND`

| Subcommand       | What it does                                                                                           |
|------------------|--------------------------------------------------------------------------------------------------------|
| \<no subcommand> | Launch Win Global GPU in default mode: integrated GPU on battery power and dedicated GPU on wall power |
| `shutdown`       | Shuts down any running stance of Win Global GPU                                                        |
| `dedicated`      | Sets the preferred GPU to the dedicated GPU                                                            |
| `integrated`     | Sets the preferred GPU to the integrated GPU                                                           |
| `reset`          | Resets the preferred GPU, lets Windows decide                                                          |
| `help`           | Shows help on how it works                                                                             |
| `about`          | Shows some information about Win Global GPU                                                            |

For best results, add Win Global GPU
as [a startup program](https://support.microsoft.com/en-us/windows/add-apps-to-the-startup-page-in-settings-3d219555-bc76-449d-ab89-0d2dd6307164).

## Some disclaimers

Currently, Win Global GPU only works for apps installed on NTFS volumes. [See why below](#how-does-it-work).

Win Global GPU is only meant for systems with a dedicated _and_ integrated GPU. No idea what happens if you use this
program without them.

## How does it work?

There is no way to globally set the preferred GPU on Windows, only per exe/Windows app. So Win Global GPU scans your
system for every exe and windows app and sets them all via the registry. It uses
the [NTFS MFT](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table) to very quickly scan your
system for all exes, similar to [WizTree](https://diskanalyzer.com/).