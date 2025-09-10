# pmenu
Minimal tui-like power menu for shutdown, reboot, suspend, lock, and logout.

![./showcase.png]

To compile it:
```bash
git clone https://github.com/gaaelgarciia/pmenu.git
cd pmenu
cargo build --release
```

To run it move to the cloned repo and execute:

```bash
./target/release/pmenu
```

The default config will be created in **.config/pmenu/**. You can change the power scripts and the bindings.

If you want to change the colorscheme, change it in the styles/ in the pmenu directory
