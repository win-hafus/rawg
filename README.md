# rawg

Stupid AmneziaWG connection manager writed on Rust

# How to install

```bash
git clone https://github.com/win-hafus/rawg
cd rawg
cargo install
```

if `~/.cargo/bin` not in $PATH do:

```bash
export $PATH=$PATH:~/.cargo/bin
```

or

```bash
echo "export $PATH=$PATH:~/.cargo/bin" >> .bashrc
```

## Warning!

If you using opendoas instead of sudo you need to add:

```conf
permit nopass <username> as root cmd awg-quick
```

# How to uninstall

```bash
cd rawg
cargo uninstall
```

or simply rm binary from `~.cargo/bin/`
