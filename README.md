# sway-scratch

Automatically starting named scratchpads for [sway](https://github.com/swaywm/sway).

## Usage

```
sway-scratch show [OPTIONS] --exec <EXEC> <--app-id <APP_ID>|--class <CLASS>>
```

Options:
- `--app-id <APP_ID>`  the Wayland app_id of the application
- `--class <CLASS>`    the window_properties.class of the application (Xwayland)
- `--exec <EXEC>`      the command to open the scratch initially
- `--resize <RESIZE>`  resize command to run when the scratch is shown (e.g. `set 90 ppt 90 ppt`)
- `-h`, `--help`

### Examples

To toggle a python terminal:

```
sway-scratch show --app-id scratch_py --exec "kitty --class scratch_py -e python -q"
```

In your `sway/config`, you would put something like the following:

```i3
set $scratchpy scratch_py
for_window [app_id=$scratchpy] move scratchpad, scratchpad show
bindsym $mod+equal sway-scratch show $scratchpy "$term --class $scratchpy -e python -q"
```

As can be seen in the example above, terminal emulators such as [kitty](https://github.com/kovidgoyal/kitty) may include
an option for you to manually set the `app_id`
([--class](https://sw.kovidgoyal.net/kitty/invocation/#cmdoption-kitty-class) in this example).

See [my personal dotfiles](https://github.com/aokellermann/dotfiles/blob/master/.config/sway/config)
for a few examples of various applications.

`swaymsg -r -t get_tree` is a useful tool to view the `sway` tree in order to verify
identifiable information about windows.

## Installation

### AUR

`sway-scratch` is available as an AUR package:

```console
yay -S sway-scratch
```

### Manual

You can build the binary yourself:

```console
git clone git@github.com:aokellermann/sway-scratch.git
cd sway-scratch
cargo build --release
```

and install it:

```console
cargo install --path crates/sway-scratch
```
