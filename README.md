# promptress

A lightning-fast, customizable prompt

![promptress](https://www.jtai.ca/files/promptress.png)

Having a very fast prompt is more important than you might think. If your prompt takes half a second to render, then you must wait for it before you can type your next command. These half-seconds can add up.

Fortunately, promptress takes **less than 5 ms** to render (but YMMV).

## Building and installing

You can install this project with Cargo:

```console
$ cargo install --path .
```

Or, you can just build it:

```console
$ cargo build --release
```

Then the binary will be at `./target/release/promptress`.

## Getting started

First, you will need a configuration file for promptress. You can start off with just an empty file (which is a valid config) and add more to it later.

```shell
$ > ~/.promptress.toml
```

Now, you can set your `$PS1`:

```shell
PS1='$(PROMPTRESS_EXIT_CODE=$? promptress)'
```

You can add the above line to your `~/.bashrc` to make the change permanent.

To tweak the colours, add a Git branch display, and more, see [**Configuration guide**](https://github.com/j-tai/promptress/wiki/Configuration-guide).

## License

[**GNU General Public License version 3**](LICENSE).
