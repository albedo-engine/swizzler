<h1 style="text-align: center">Swizzler</h1>

## Installation

**Swizzler!** isn't available yet on [crates.io](https://crates.io).

However, installing and running it is straighforward using [cargo](https://doc.rust-lang.org/cargo). Depending on your use case, you have several options
available.

#### 1. Install binary from sources

You can download, build and install locally the CLI using:

```sh
$ cargo install --git https://github.com/albedo-engine/swizzler.git
```

Check that the installation was successful:

```sh
$ swizzler --version
```

#### 2. Install library as a dependency

**Swizzler!** can also be used programmatically. If you plan on using the
library, you can link it to your dependency by modying your `Cargo.toml` file:

```toml
[dependencies]
swizzler = { git = "https://github.com/albedo-engine/swizzler.git" }
```
## CLI Usage

**Swizzler!** CLI can be used either to generate a texture after cherry picking
channels of multiple sources, or it can be run on an entire folder automatically.

### Manual Swizzling

You can generate manually a new texture by providing, for each texture source,
which channel to extract.

The CLI would look something like:

```sh
$ swizzler --input red --input green --input blue --input alpha
```

Each channel (`red`, `green`, `blue`, and `alpha`) should be set to a texture, e.g:

```sh
$ swizzler --input ./texture_1.png --input ./texture_2.png ...
```

The position of each `--input` argument select the output channel. In order to
select the source channel for each input image, you have to specify it at the
end of the source path, delimited by the `:` character:

```sh
$ swizzler --input ./texture_1.png:2 --input ./texture_2.png:0
```

Let's have a look at an example, using the input file `source.png`:

![](TODO)

Let's shuffle all channels as follows:
* The **red** channel becomes the **blue** channel
* The **green** channel becomes the **red** channel
* The **blue** channel becomes the **green** channel

```sh
$ swizzler manual --input ./source.png:b --input ./source.png:r ./source.png:g
```

And the result is:

![](TODO)

### Swizzling Folder

Sometimes, you need to process an entire hierarchy. Using the [Manual Command](#manual-swizzling) is handy, but can turn especially difficult when you need
to find what files should be grouped together.

This is why **Swizzler!** comes with a second command: `session`.

The `session` command let you use an advanced JSON configuration file containing
the files to resolve together, and the textures to generate with those files. Let's
have a look at a config file example:

```json
{
  "base": "(.*)_.*",
  "matchers": [
      { "id": "metalness", "matcher": "(?i)metal(ness)?" },
      { "id": "roughness", "matcher": "(?i)rough(ness)?" },
      { "id": "albedo", "matcher": "(?i)albedo" }
  ],
  "targets": [
    {
      "name": "-metalness-roughness.png",
      "output_format": "png",
      "inputs": [
          [ "metalness", 0 ],
          null,
          null,
          [ "roughness", 0 ]
      ]
    }
  ]
}
```

Let's take a look at a real life example. We have a `textures` folder containing
the following:

```sh
$ ls ./textures
enemy_albedo.png    enemy_metalness.png enemy_roughness.png hero_albedo.png     hero_metalness.png  hero_roughness.png
```

#### `base` attribute

The `base` attribute describe how to extract the name of the asset from a path.
This **has to be** a [Regular Expression](https://en.wikipedia.org/wiki/Regular_expression) with **one** capturing group. In this examplee, the base
captures everything before the last `_` character. All the files starting with
`hero_` would have the base `hero`, and all the files starting with `enemy_` the
base `enemy`.

#### `matchers` attribute

The matchers provide a list of files to match under the same asset. In this
example, all the metalness, roughness, and albedo textures belonging to a same
asset would get resolved.

#### `targets` attributes
