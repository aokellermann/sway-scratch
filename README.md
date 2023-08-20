# sway-scratch

## Usage


```
sway-scratch show [NAME] [EXEC]
```

For example, in your sway config, to have a named python terminal:

```i3
set $scratchpy scratch_py
for_window [app_id=$scratchpy] move scratchpad; scratchpad show
bindsym $mod+equal sway-scratch show $scratchpy "$term --class $scratchpy -e python -q"
```
