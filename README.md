# COSMIC Logo Menu

A simple logo menu (in the style of the popular [GNOME extension](https://github.com/Aryan20/Logomenu)) to combine your launcher options in a nice menu.

![Screenshot_2025-05-05_23-10-06](https://github.com/user-attachments/assets/f03d5699-b04d-4802-87d4-6c38ae9ecd3e)

This is in a very early state and currently has no customisation available. The plan for features are:

* A better and more comprehensive solution for settings (currently you can only update the logo from within the menu itself)
* Add / remove custom options
* Hide / show default options
* Reorder the menu and dividers among custom and default options as you please

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
