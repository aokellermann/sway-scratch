# sway-scratch

Automatically starting named scratchpads for sway.

## Usage

```
sway-scratch show [NAME] [EXEC]
```

where:

- `NAME` is the Wayland `app_id` of the application
- `EXEC` is the command to open the scratch initially

For example, in your sway config, to have a named python terminal:

```i3
set $scratchpy scratch_py
for_window [app_id=$scratchpy] move scratchpad; scratchpad show
bindsym $mod+equal sway-scratch show $scratchpy "$term --class $scratchpy -e python -q"
```

As can be seen in the example above, terminal emulators such as [kitty](https://github.com/kovidgoyal/kitty) may include
an option for you to manually set the `app_id`
([--class](https://sw.kovidgoyal.net/kitty/invocation/#cmdoption-kitty-class) in this example).

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
install -Dm755 target/release/sway-scratch -t /usr/bin/
install -Dm644 LICENSE -t /usr/share/licenses/sway-scratch/
```
