# OGJ Ferium

[![rust badge](https://img.shields.io/static/v1?label=Made%20with&message=Rust&logo=rust&labelColor=e82833&color=b11522)](https://www.rust-lang.org)
[![licence badge](https://img.shields.io/github/license/OgGhostJelly/ferium)](https://github.com/OgGhostJelly/ferium/blob/main/LICENSE.txt)
[![build.yml](https://github.com/OgGhostJelly/ferium/actions/workflows/build.yml/badge.svg)](https://github.com/OgGhostJelly/ferium/actions/workflows/build.yml)

> Check out ferium's sister projects [ferinth](https://github.com/gorilla-devs/ferinth) and [furse](https://github.com/gorilla-devs/furse).
> They are Rust wrappers for the official Modrinth and CurseForge APIs respectively.

> [!WARNING]
> Expect bugs and breaking changes.
> For legitimate use-cases consider using the [official Ferium](https://github.com/gorilla-devs/ferium).

OGJ Ferium is my customized version of Ferium. You can migrate your Ferium configs to ogj-ferium with the command `ogj-ferium migrate /path/to/config.json`. The migration may fail but it should work for most cases.

Ferium is a fast and feature rich CLI program for downloading and updating Minecraft mods, modpacks, resourcepacks and shaderpacks from [Modrinth](https://modrinth.com), [CurseForge](https://curseforge.com/minecraft), and [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases).
Simply specify the mods you use, and in just one command you can download the latest compatible version of all the mods you configured.

Key differences from Ferium:
- Profiles can be separate files which allows for easier distribution of profiles or alternatively embedded into the config file like the original Ferium.
- The filters system was replaced and the format is now TOML.
- Resourcepacks and shaderpacks are supported, which is not yet available in Ferium. See [#141](https://github.com/gorilla-devs/ferium/issues/141)

An example of an ogj-ferium profile can be found [here](./examples/example.toml)

## Features

- Use the CLI to easily automate your modding experience
- Download from multiple sources, namely [Modrinth](https://modrinth.com), [CurseForge](https://curseforge.com/minecraft), and [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
- <details>
    <summary>Beautiful and informative UI</summary>

    #### Profile info and listing mods
    ![Profile Information and Listing Mods](media/profile_info_and_list.png)

    #### Listing mods verbosely
    ![Listing Mods Verbosely](media/list_verbose.png)

    #### Upgrading mods/modpacks
    ![Upgrading Mods/Modpacks](media/upgrade.png)
  </details>

- <details>
    <summary>It's super fast due to multithreading for network intensive tasks</summary>

    Your results may vary depending on your internet connection.

    It downloads theRookieCoder's modpack [Kupfur](https://github.com/theRookieCoder/Kupfur) with 79 mods in 15 seconds:

    https://github.com/OgGhostJelly/ferium/assets/60034030/cfe2f0aa-3c10-41ca-b223-367925309ea9

    It downloads [MMTP](https://www.curseforge.com/minecraft/modpacks/mats-mega-tech-pack), a very large modpack with around 400 mods, in just under a minute:

    https://github.com/OgGhostJelly/ferium/assets/60034030/857e8d27-372d-4cdd-90af-b0d77cb7e90c
  </details>

- Upgrade all your mods to the latest compatible version in one command, `ogj-ferium upgrade`
  - Ferium checks that the version being downloaded is the latest one that is compatible with the configured mod loader and Minecraft version
- Create multiple profiles and configure different mod loaders, Minecraft versions, output directories, and mods for each

## Installation

Ferium executables from GitHub Releases do not require any external dependencies at runtime.  
If you compile from source on Linux, using GCC to build will result in binaries that require GCC to be available at runtime.  
On Linux, the regular versions require some sort of desktop environment to be available that offers an XDG Desktop Portal to show the folder picker.
The `nogui` versions do not need this as they won't have a GUI folder picker, making these variants suitable for server use.

> [!IMPORTANT]
> Linux users! Use the `nogui` versions (or compile with `--no-default-features`) if you do not have a desktop environment (like GNOME, KDE, XFCE, etc.)

### Packages

> [!NOTE]
> Only Github Releases is supported for ogj-ferium.
> This is because maintaining packages is hard and this isn't a commercial product.

#### GitHub Releases
[![GitHub Releases](https://img.shields.io/github/v/release/OgGhostJelly/ferium?color=bright-green&label=github%20releases)](https://github.com/OgGhostJelly/ferium/releases)
> [!IMPORTANT]
> You will have to manually download and install every time there is a new update.

1. Download the asset suitable for your operating system from the [latest release](https://github.com/OgGhostJelly/ferium/releases/latest)
2. Unzip the file and move it to a folder in your path, e.g. `~/bin`
3. Remember to check the releases page for any updates!

## Overview / Help Page

> [!NOTE]
> A lot of ferium's backend is in a separate project; [libium](https://github.com/OgGhostJelly/libium).  
> It deals with things such as the config, adding mod(pack)s, upgrading, file pickers, etc.

### Program Configuration

Ferium stores profile information in its config file. By default, this is located at `~/.config/ferium/ogj-config.toml`.  
You can change this in 2 ways, setting the `OGJ_FERIUM_CONFIG_FILE` environment variable, or setting the `--config-file` global flag.
The flag always takes precedence.

> [!CAUTION]
> Be mindful of syntax when manually editing the config file

You can also set a custom CurseForge API key or GitHub personal access token using the `CURSEFORGE_API_KEY` and `GITHUB_TOKEN` environment variables, or the `--curseforge_api_key` and `--github-token` global flags respectively.
Again, the flags take precedence.

### First Startup

[Create a new profile](#creating) by running `ogj-ferium profile create` and entering the details for your profile.
- Then, [add your mods](#manually-adding-mods) using `ogj-ferium add`.
- Finally, download your mods using `ogj-ferium upgrade`.

### Automatically Import Mods

```bash
ferium scan
```

This command scans a directory with mods, and attempts to add them to your profile.

The directory defaults to your profile's output directory. Some mods are available on both Modrinth and CurseForge; ferium will prefer Modrinth by default, but you can choose CurseForge instead using the `--platform` flag.

As long as you ensure the mods in the directory match the configured mod loader and Minecraft version, they should all add properly. Some mods might require some [additional tuning](#check-overrides). You can also bypass the compatibility checks using the `--force` flag.

### Manually Adding Mods

> [!TIP]
> You can specify multiple identifiers to add multiple mods at once

#### Modrinth
```bash
ferium add project_id
```

`project_id` is the slug or project ID of the mod. (e.g. [Sodium](https://modrinth.com/mod/sodium) has the slug `sodium` and project ID `AANobbMI`). You can find the slug in the website URL (`modrinth.com/mod/<slug>`), and the project ID at the bottom of the left sidebar under 'Technical information'.  
So to add [Sodium](https://modrinth.com/mod/sodium), you can run `ogj-ferium add sodium` or `ogj-ferium add AANobbMI`.

#### CurseForge
```bash
ferium add project_id
```
`project_id` is the project ID of the mod. (e.g. [Terralith](https://www.curseforge.com/minecraft/mc-mods/terralith) has the project id `513688`). You can find the project id at the top of the right sidebar under 'About Project'.  
So to add [Terralith](https://www.curseforge.com/minecraft/mc-mods/terralith), you should run `ogj-ferium add 513688`.

#### GitHub
```bash
ferium add owner/name
```
`owner` is the username of the owner of the repository and `name` is the name of the repository, both are case-insensitive (e.g. [Sodium's repository](https://github.com/CaffeineMC/sodium) has the id `CaffeineMC/sodium`). You can find these at the top left of the repository's page.  
So to add [Sodium](https://github.com/CaffeineMC/sodium), you should run `ogj-ferium add CaffeineMC/sodium`.

> [!IMPORTANT]
> The GitHub repository needs to upload JAR files to their _Releases_ for ferium to download, or else it will refuse to be added.

#### User Mods

If you want to use files that are not downloadable by ferium, place them in a subfolder called `user` in the output directory. Files here will be copied to the output directory when upgrading.

> [!NOTE]
> Profiles using Quilt will not copy their user mods, this is because Quilt automatically loads mods from nested directories (such as the user folder) since version `0.18.1-beta.3`.

### Upgrading Mods

> [!WARNING]
> If your output directory is not empty when setting it, ferium will offer to create a backup.  
> Please do so if it contains any files you would like to keep.

Now after adding all your mods, run `ogj-ferium upgrade` to download all of them to your output directory.
This defaults to `.minecraft/mods`, where `.minecraft` is the default Minecraft resources directory. You don't need to worry about this if you play with Mojang's launcher and use the default resources directory.
You can choose to pick a custom output directory during profile creation or [change it later](#configure-1).

If ferium fails to download a mod, it will print its name in red and try to give a reason. It will continue downloading the rest of your mods and will exit with an error.

> [!TIP]
> When upgrading, any files not downloaded by ferium will be moved to the `.old` folder in the output directory.  
> See [user mods](#user-mods) for information on how to add mods that ferium cannot download.

### Managing Mods

You can list out all the mods in your current profile by running `ogj-ferium list`. If you want to see more information about them, you can use `ogj-ferium list -v` or `ogj-ferium list --verbose`.

You can remove any of your mods using `ogj-ferium remove`; just select the ones you would like to remove using the space key, and press enter once you're done. You can also provide the names, IDs, or slugs of the mods as arguments.

> [!TIP]
> Older versions did not have the ability to remove mods by their slug, since it wasn't stored in the config. If you are upgrading from an older version, you will initially not be able to remove mods by their slugs. You can run `ogj-ferium list -v` to load the slugs into the profile.

> [!IMPORTANT]
> Both mod names and GitHub repository identifiers are case insensitive.  
> Mod names with spaces have to be given in quotes (`ogj-ferium remove "ok zoomer"`) or the spaces should be escaped (usually `ogj-ferium remove ok\ zoomer`, but depends on the shell).

#### Check Overrides

If some mod is supposed to be compatible with your game version and mod loader, but ferium does not download it, [create an issue](https://github.com/OgGhostJelly/ferium/issues/new?labels=bug&template=bug-report.md) if you think it's a bug.

If you suspect the author has not specified compatible versions or mod loaders, you can disable the game version or mod loader checks by using the `--ignore-game-version` and/or `--ignore-mod-loader` flags when adding a single mod, or manually setting `check_game_version` and/or `check_mod_loader` to false for the specific mod in the config file.

For example, [Just Enough Items](https://www.curseforge.com/minecraft/mc-mods/jei) does not specify the mod loader for older Minecraft versions such as `1.12.2`. In this case, you would add JEI by running `ogj-ferium add 238222 --ignore-mod-loader` so that the mod loader check is disabled.  
You can also manually disable the mod loader (and/or game version) check(s) in the config like so:
```json
{
    "name": "Just Enough Items (JEI)",
    "identifier": {
        "CurseForgeProject": 238222
    },
    "check_mod_loader": false
}
```

### Profiles

#### Creating

You can create a profile by running `ogj-ferium profile create` and specifying the following:

- Output directory
  - This defaults to `.minecraft/mods` where `.minecraft` is the default Minecraft resources directory. You don't need to worry about this if you play with Mojang's launcher and use the default resources directory.
- Name of the profile
- Minecraft version
- Mod loader

If you want to copy the mods from another profile, use the `--import` flag.
If you want to embed the profile in the config file, use the `--embed` flag.
You can also directly provide the profile name to the flag if you don't want a profile picker to be shown.

> [!NOTE]
> Ferium will automatically switch to the newly created profile

> [!TIP]
> You can also provide these settings as flags to avoid interactivity for things like scripts


#### Configure

You can configure these same settings afterwards by running `ogj-ferium profile configure`. Again, you can provide these settings as flags.

#### Manage

You can get information about the current profile by running `ogj-ferium profile` or `ogj-ferium profile info`, and about all the profiles you have by running `ogj-ferium profiles` or `ogj-ferium profile list`.  
Switch to a different profile using `ogj-ferium profile switch`.  
Delete a profile using `ogj-ferium profile delete` and selecting the profile you want to delete.

## Feature Requests

If you would like to make a feature request, check the [issue tracker](https://github.com/OgGhostJelly/ferium/issues?q=is%3Aissue+label%3Aenhancement) to see if the feature has already been added or is planned.
If not, [create a new issue](https://github.com/OgGhostJelly/ferium/issues/new/choose).

## Developing

Firstly, you will need the Rust toolchain, which includes `cargo`, `rustup`, etc. You can install these [using rustup](https://www.rust-lang.org/tools/install).
You can manually run cargo commands if you wish, but I recommend using the `justfile` configuration in the repository. [`just`](https://just.systems/man/en) is a command runner that is basically a much better version of `make`.

To build the project and install it to your Cargo binary directory, clone the project and run `just install`.
If you want to install it for testing purposes, [add the nightly toolchain](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html#rustup-and-the-role-of-rust-nightly) and run `just` (aliased to `just install-dev`), which has some optimisations to make compilation faster.

You can run integration tests using `cargo test`, lint using `cargo clippy`, and delete all build and test artefacts using `just clean`.

If you would like to see how to cross-compile for specific targets (e.g. Linux ARM) or other information such as the development libraries required, have a look at the [workflow file](.github/workflows/build.yml).  
If you still have doubts, feel free to [create a discussion](https://github.com/OgGhostJelly/ferium/discussions/new?category=q-a) and I will try help you out.
