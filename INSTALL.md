# Local build

Building papers locally require the following system dependencies:

```
sudo apt install libpoppler-glib-dev libpoppler-dev libicu-dev
```

Poppler is a cairo-based PDF rendering library, and is used for PDF rendering inside the APP. 
ICU is required by tectonic (the Rust-based Latex engine wrapper used by drafts). 

# Flatpak build

Perhaps you want to use a flatpak SDK instead of the system toolchain. This is perhaps the easiest approach,
since both poppler and libicu will be bundled, and you don't have to worry about them. 

If you don't have flatpak installed yet:

```
sudo apt install flatpak
```

Just make sure to have
the flatpak SDK and rust toolchain extension installed (note that this step will be done automatically if you
just run flatpak build [drafts]:

```
flatpak install org.gnome.Sdk org.gnome.Platform org.freedesktop.Sdk.Extension.rust-stable
```

When using the flatpak build, poppler will be bundled. TODO figure how to bundle libicuuc, required by tectonic/crates/bridge_icu

Workaround: Copy system libicuuc
cp /usr/lib/x86_64-linux-gnu/libicuuc.so.67.1 /home/diego/Downloads/papers-build/build/files/bin
mv /home/diego/Downloads/papers-build/build/files/bin/libicuuc.so.67.1 /home/diego/Downloads/papers-build/build/files/bin/libicuuc.so.69
LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/home/diego/Downloads/papers-build/build/files/bin ./papers

## To build the application into a custom directory

This exports the executable to the repo folder, and leave build artifacts at the build folder.

```
flatpak-builder --repo=/home/diego/Downloads/papers-build/repo \
    /home/diego/Downloads/papers-build/build com.github.limads.Papers.json \
    --state-dir=/home/diego/Downloads/papers-build/state --force-clean
```

(This will leave a lot of artifacts at state dir (replacement for .flatpak-builder at current dir), which will be created at the directory the command is called).

## To install locally

This installs the executable to the local flatpak applications directory

flatpak-builder --install /home/diego/Downloads/papers-build/build com.github.limads.Papers.json --force-clean

The flatpak build output will result in three directories: bin (with the papers executable), lib (with libpoppler.so and libpoppler-glib.so) and share (with appdata/com.github.limads.Papers.appdata.xml, app-info/icons and app-info/xmls, applications/com.github.limads/Papers.desktop, glib-2.0/schemas/(gschema files) and icons/hicolor/scalable and icons/hicolor/symbolic)
Local install will be at `~/.local/share/flatpak/` (user) or `/var/lib/flatpak/repo` (system)

Clean with `flatpak uninstall com.github.limads.Papers && flatpak uninstall --unused` (The second command will uninstall Gnome 42 SDK when not by other apps).

To verify the checksum of the libicu zip:

```
sha256sum icu4c-69_1-src.zip
```