# PunktF - A cross-platform multi-target dotfiles manager

[![MIT License](https://img.shields.io/crates/l/punktf)](https://choosealicense.com/licenses/mit/) [![Continuous integration](https://github.com/Shemnei/punktf/workflows/Continuous%20Integration/badge.svg)](https://github.com/Shemnei/punktf/actions) [![Crates.io](https://img.shields.io/crates/v/punktf)](https://crates.io/crates/punktf)

## DISCLAIMER

This crate is sill under development and not all features are currently implemented.
Layouts and formats can and will change while in development.

The following features are already implemented:

- [x] Basic deployment process
- [x] Basic templating support
- [x] Reading from a profile file
- [x] Depolying non template items
- [x] Directories can be used as an item
- [x] Profiles can have another profile as a base
- [x] Pre/Post deployment hooks
- [x] Basic support for merge operations

Before you try this tool, please make sure to save/backup your existing setup.
While in deployment there are likely bugs which can and will mess up your
existing setup.

## Yet another dotfile manager?!

Well yes, but hear me out. This project was driven by the personal need of having to manage several dotfiles for different machines/targets. You want the same experience everywhere: On your work Windows machine along with an Ubuntu WSL instance, your Debian server and your private Arch installation. This tool fixes that problem while beeing cross-platform and blazingly fast.

Features:
- [ ] Merge mutliple layers of dotfiles
- [ ] Create profiles for different targets
- [ ] Use instructions to compile your dotfiles/templates conditionally
- [ ] Use hadlebar-like instructions to insert variables and more
- [ ] Define pre- and post-hooks to customize the behaviour with custom commands
- [ ] Handles file permissions and line endings (CRLF vs LF)

## Commands

```shell
# deploy (dry-run)
punktf deploy windows --dry-run

# deploy (custom source folder)
punktf --source /home/demo deploy windows

# deploy (custom home folder)
PUNKTF_SOURCE=/home/demo punktf deploy windows
```

## PunktF Source

PunktF searches for the source path in the following order:

1) CLI argument given with `-s/--source`
2) Environment variable `PUNKTF_SOURCE`
3) Current working directory of the shell

```
+ profiles\
	+ windows.pfp
+ items\
	+ init.vim.win
```

## PunktF profile (either json or yaml)

```json5
{
	// OPT: Other profile which will be used as base for this one
	"extends": "base_profile_name",

	// OPT: Variables for all items
	"vars": [
		{
			"key": "RUSTC_PATH",
			"value": "/usr/bin/rustc",
		}
		//, ...
	],

	// Target path of config dir; used when no specific deploy_location was given
	"target": "/home/demo/.config",

	// OPT: Hooks which are executed once before the deployment.
	"pre_hooks": ["echo \"Foo\""],

	// OPT: Hooks which are executed once after the deployment.
	"post_hooks": ["echo \"Bar\""],

	// Items to be deployed
	"items": [
		{
			// Relative path in `items/`
			"path": "init.vim.win",

			// OPT: Alternative deploy target (PATH: used instead of `root` + `file`, ALIAS: `root` + (alias instead of `file`))
			"target": {
				"kind": "alias",
				"value": "init.vim",
			},

			// OPT: Custom variables for the specific file (same as above)
			"vars": [
				...
			],

			// OPT: Merge operation/kind (like: overwrite_all, ask, keep, overwrite_deployed)
			"merge": "overwrite",

			// OPT: Wether this file is a template or not (skips template actions (replace, ..) if not)
			"template": false,

			// OPT: Higher priority item is allowed to overwrite lower priority ones
			"priority": 2,
		}
		//, ...
	]
}
```

## Template Format

### Replacement

Prefix (can be combined: e.g. {{#$RUSTC_PATH}}):

- None: First profile.env then profile.file.env
- `$`: Only ENVIRONMENT
- `#`: Only env
- `&`: Only file.env

```python
rustc = {{RUSTC_PATH}}
```

### Conditionals (TODO: think about structure)

```python
{{@if {{OS}} == "windows"}}
	print("running on windows")
{{@elif {{OS}} == "linux"}}
	print("running on linux")
{{@else}}
	print("NOT running on windows/linux")
{{@fi}}
```
