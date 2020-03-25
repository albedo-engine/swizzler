<h1 align="center">Swizzler</h1>

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

### Manual

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

![](./textures/rgb.png | =250x250)

### Folder

Sometimes, you need to process an entire hierarchy. Using the [Manual Command](#manual) is handy, but can turn especially difficult when you need
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

The targets array makes use of the `matchers` list in order to know what textures
to use as sources. Each target will generate exaclty one texture, containing the
combination of specificied sources.

Here, we use the `metalness` and `roughness` identifiers to specify to create
a new texture, containing **4** channels. The `red` channel will be filled with
the metalness texture `red channel`, and the `alpha` channel will be filled with
the roughness texture `red channel`.

The `name` attribute allows you to customize the name used when saving the file,
and the `output_format` allows you to specify an [encoding format](#arguments).

You are now ready to run the session:

```sh
$ swizzler session --folder ./textures --config ./config.json
```

Alternatively, you can provide the `config.json` file on `stdin`:

```sh
$ cat ./config.json | swizzler session --folder ./textures
```

The results should be generated in the folder `__swizzler_build`, as follows:

```sh
$ ls ./__swizzler_build
enemy-metalness-roughness.png hero-metalness-roughness.png
```

### Arguments

#### Manual command

Usage:

```sh
$ swizzler manual [-i PATH] ... [-i PATH]
```

|Argument|Value|Description|
|:--:|:--:|:--------------------|
|**-o, --output**|_Path_|Relative path to which output the texture|
|**-i, --input**|_Path_|Relative path to the texture source to use|

#### Session command

Usage:

```sh
$ swizzler session --folder PATH [--config PATH_TO_CONFIG]
```

|Argument|Value|Description|
|:--:|:--:|:--------------------|
|**-f, --folder**|_Path_|Relative path to the folder to process|
|**-o, --output**|_[Path]_|Relative path to the folder in which to output files|
|**-c, --config**|_[Path]_|Relative path to the config to use|
|**-n, --num_threads**|_[Number]_|Number of threads to use. Default to the number of logical core of the machine|

For the output format, here is a list of all available encoding format.

* `png`
* `jpg`
* `tga`
* `pnm`
* `gif`
* `ico`
* `bmp`
