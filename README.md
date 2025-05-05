# COSMIC Logo Menu

A simple logo menu (in the style of the popular GNOME extension) to combine your launcher options in a nice menu.

This is in a very early state and currently has no customisation available. The plan for features are:

* Configurable distro logo (currently defaults to COSMIC, the thing that unites us all)
* Hide / Show existing default launcher options
* Complete customisation to add custom launchers, dividers and reorder everything to your liking

## Install

To install, you will need [just](https://github.com/casey/just), if you're on Pop!\_OS, you can install it with the following command:

```sh
sudo apt install just
```

After you install it, you can run the following commands to build and install the applet:

```sh
just build-release
sudo just install
```
