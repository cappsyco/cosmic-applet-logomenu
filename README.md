# COSMIC™ Logo Menu

A simple logo menu (in the style of the popular [GNOME extension](https://github.com/Aryan20/Logomenu)) to collect your launcher options in a nice menu.

![A view of the open applet with the default menu items, alongside the included settings application.](https://raw.githubusercontent.com/cappsyco/cosmic-applet-logomenu/main/screenshots/cosmic-applet-logomenu.png)

## Installing

Two binaries are currently required for the Logo Menu to be fully functional (one for the applet itself and a separate settings app). You can get these binaries from the [latest release](https://github.com/cappsyco/cosmic-applet-logomenu/releases/latest) and install using the following instructions.

### .deb distros (Pop!\_OS, Debian etc.)

```sh
sudo dpkg -i cosmic-applet-logomenu_0.5.0_amd64.deb
sudo dpkg -i cosmic-logomenu-settings_0.5.0_amd64.deb
```

### .rpm distros (Fedora etc.)

```sh
sudo dnf install cosmic-applet-logomenu-0.5.0-1.fc42.x86_64.rpm
sudo dnf install cosmic-logomenu-settings-0.5.0-1.fc42.x86_64.rpm
```

### Arch based distros

The applet and settings app can be installed directly from [one package in the AUR](https://aur.archlinux.org/packages/cosmic-applet-logomenu). You will need `base-devel` and `git` if you don't have them already.

```sh
sudo pacman -S base-devel git
git clone https://aur.archlinux.org/cosmic-applet-logomenu.git
cd cosmic-applet-logomenu && makepkg -si
```

## With thanks & Credit
* [System76 and their COSMIC desktop environment](https://system76.com/cosmic/)
* [COSMIC Utilities](https://github.com/cosmic-utils/) - Organization containing third party utilities for COSMIC™
* [Logo Menu](https://github.com/Aryan20/Logomenu) by Aryan20 - For the inspiration and being a fantastic resource for the logos used here
* [Hand Pointer icon used in the logo](https://www.svgrepo.com/svg/430337/line-hand-pointer-event) by [Liny Tiny Icons](https://www.svgrepo.com/collection/liny-tiny-line-icons/)
