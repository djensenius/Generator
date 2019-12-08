# Generator
Code related to running a sine tone generator on a Raspberry Pi.

Note: `SuperCollider/startup.scd` is a combination of generator and synths manually combined for a startup file. If you make changes to either of these files make sure to copy those changes to startup file.

## Tips

### SuperCollider
Copy or link `SuperCollider/startup.scd` and `SuperCollider/cpuInfo.sh` into `.config/SuperCollider/`

### Autostart SuperCollider and disable screensaver
Edit `/etc/xdg/lxsession/LXDE/autostart` and `/etc/xdg/lxsession/LXDE-pi/autostart`

Put the following
```
@lxpanel --profile LXDE
@pcmanfm --desktop --profile LXDE
# @xscreensaver -no-splash
@xset -dpms
@xset s noblank
@/usr/bin/scide
```
