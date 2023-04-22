# Changelog

This will list notable changes from release to release, and credit the people who contributed them. This mainly covers changes that are visible to end users, so please look at the commit history if you want to know all the details.

Names preceded by an @ are GitHub usernames.

Changes are categorised as follows:

* Compatibility: changes that affect which apps work in touchHLE.
* Quality and performance: changes that don't affect which apps work, but do affect the quality of the experience.
* Usability: changes to features of the emulator unrelated to the above, e.g. new input methods.
* Other: when none of the above seem to fit.

If an app is added to the supported list after the relevant version has already been released, its entry in the changelog will be followed by the date it was added \[in square brackets\].

## NEXT

Compatibility:

- API support improvements:
  - Various small contributions. (@hikari-no-yume, @KiritoDv)

Quality:

- Overlapping characters in text now render correctly. (@Xertes0)

Usability:

- touchHLE now supports real accelerometer input on devices with a built-in accelerometer, such as phones and tablets. This is only used if no game controller is connected. (@hikari-no-yume)
- The options help text is now available as a file (`OPTIONS_HELP.txt`), so you don't have to use the command line to get a list of options. (@hikari-no-yume)
- The new `--fullscreen` option lets you display an app in fullscreen rather than in a window. This is independent of the internal resolution/scale hack and supports both upscaling and downscaling. (@hikari-no-yume)

Other:

- touchHLE now has a primitive implementation of the GDB Remote Serial Protocol. GDB can connect to touchHLE over TCP and set software breakpoints, inspect memory and registers, step or continue execution, etc. This replaces the old `--breakpoint=` option, which is now removed. (@hikari-no-yume)
- The version of SDL2 used by touchHLE has been updated to 2.26.4. (@hikari-no-yume)
- Building on common Linux systems should now work without problems, and you can use dynamic linking for SDL2 and OpenAL if you prefer. Note that we are not providing release binaries. (@GeffDev)

## v0.1.2 (2023-03-07)

Compatibility:

- API support improvements:
  - Various small contributions. (@hikari-no-yume, @nitinseshadri)
  - Some key parts of `UIImage`, `CGImage` and `CGBitmapContext` used by Apple's `Texture2D` sample code are now implemented. Loading textures from PNG files in this way should now work. (@hikari-no-yume)
  - MP3 is now a supported audio file format in Audio Toolbox. This is done in a fairly hacky way so it might not work for some apps. (@hikari-no-yume)
- New supported apps:
  - Touch & Go LITE
  - Touch & Go \[2023-03-12\]
  - Super Monkey Ball Lite (full version was already supported)

Quality:

- The version of stb\_image used by touchHLE has been updated. The new version includes a fix for a bug that caused many launch images (splash screens) and icons to fail to load. Thank you to @nothings and @rygorous who diagnosed and fixed this.

Usability:

- The virtual cursor controlled by the right analog stick now uses a larger portion of the analog stick's range. (@hikari-no-yume)
- Basic information about the app bundle, such as its name and version number, is now output when running an app. There is also a new command-line option, `--info`, which lets you get this information without running the app. (@hikari-no-yume)
- You are now warned if you try to run an app that requires a newer iPhone OS version. (@hikari-no-yume)
- Options can now be loaded from files. (@hikari-no-yume)
  - The recommended options for supported apps are now applied automatically. See the new `touchHLE_default_options.txt` file.
  - You can put your own options in the new `touchHLE_options.txt` file.
  - If you're a Windows user, this means that dragging and dropping an app onto `touchHLE.exe` is now all you need to do to run an app.

Other:

- The version of dynarmic used by touchHLE has been updated. This will fix build issues for some people. (@hikari-no-yume)

## v0.1.1 (2023-02-18)

Compatibility:

- API support improvements:
  - Various small contributions. (@hikari-no-yume, @nitinseshadri, @LennyKappa, @RealSupremium)
  - Basic POSIX file I/O is now supported. Previously only standard C file I/O was supported. (@hikari-no-yume)
  - Very basic use of Audio Session Services is now supported. (@nitinseshadri)
  - Very basic use of `MPMoviePlayerController` is now supported. No actual video playback is implemented. (@hikari-no-yume)
- New supported app: Crash Bandicoot Nitro Kart 3D (version 1.0 only).

Quality and performance:

- The code that limits CPU use has reworked in an attempt to more effectively balance responsiveness and energy efficiency. Frame pacing should be more consistent and slowdowns should be less frequent. No obvious impact on energy use has been observed. (@hikari-no-yume)
- The emulated CPU can now access memory via a more direct, faster path. This can dramatically improve performance and reduce CPU/energy use, in some cases by as much as 25%. (@hikari-no-yume)
- Fixed missing gamma encoding/decoding when rendering text using `UIStringDrawing`. This was making the text in _Super Monkey Ball_'s options menu look pretty ugly. (@hikari-no-yume)

Usability:

- `.ipa` files can now be opened directly, you don't need to extract the `.app` first. (@DCNick3)
- New command-line options `--landscape-left` and `--landscape-right` let you change the initial orientation of the device. (@hikari-no-yume)
- The app bundle or `.ipa` file no longer has to be the first command-line argument. (@hikari-no-yume)

Other:

- Some of the more spammy warning messages have been removed or condensed. (@hikari-no-yume)

## v0.1.0 (2023-02-02)

First release.
