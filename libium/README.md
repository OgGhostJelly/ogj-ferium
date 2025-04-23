# OGJ Libium
OGJ Libium is my own customized backend of [ogj-ferium](https://github.com/OgGhostJelly/ferium). It helps manage Minecraft mods from Modrinth, CurseForge, and Github Releases.

These are the main components of libium;

- `config` deals with (surprise, surprise) the config. It defines the config structure and methods to get the config file, deserialize it, etc
- `modpack` contains manifest/metadata structs for MR and CF modpack formats, and functions for reading these from a zip file
- `upgrade` contains functions for fetching the latest compatible mod/modpack file, and downloading it
- `add` contains functions to verify and add a mod to a profile
- `file_picker` contains functions to show a file picker for both GUI and CLI styles
