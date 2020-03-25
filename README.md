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
$ swizzler manual [--i red] [--i green] [--i blue] [--input a]
```

Each channel (`red`, `green`, `blue`, and `alpha`) should be set to a texture, e.g:

```sh
$ swizzler manual --i ./texture_1.png --i ./texture_2.png ...
```

The position of each `--input` argument select the output channel. In order to
select the source channel for each input image, you have to specify it at the
end of the source path, delimited by the `:` character:

```sh
$ swizzler manual --input ./texture_1.png:2 --input ./texture_2.png:0
```

The number of arguments determines the number of channels of the output image. If
only one `--input` is given, the image will be saved as _Grayscale__. If four
`--input` are set, the image will be saved as _RGBA_. If you don't care about
a particular channel, you can let it empty by setting it to `none`, e.g:

```sh
$ swizzler manual -i red.png:0 -i none -i none -i alpha.png:3
```

### Folder

Sometimes, you need to process an entire hierarchy. Using the [Manual Command](#manual) is handy, but can turn especially difficult when you need to find what files should be grouped together.

The `session` command let you use an advanced JSON configuration file containing
the files to resolve together, and the textures to generate with those files. Let's
have a look at a config file example:

Let's take a look at a real life example. We have a `textures` folder containing
the following:

```sh
$ ls ./textures
enemy_albedo.png    enemy_metalness.png enemy_roughness.png hero_albedo.png     hero_metalness.png  hero_roughness.png
```

And a configuration file:

```sh
$ cat ./config.json
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

#### `base` attribute

The `base` attribute describe how to extract the name of the asset from a path.
This **has to be** a [Regular Expression](https://en.wikipedia.org/wiki/Regular_expression) with **one** capturing group. In this example, the base captures everything before the last `_` character.
All the files starting with `hero_` would have the base `hero`, and all the files
starting with `enemy_` the base `enemy`.

#### `matchers` attribute

The matchers provide a list of files to match under the same asset. In this
example, the metalness, roughness, and albedo textures belonging to a same
asset will get resolved together.

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

We can now run the CLI on our `textures/` folder:

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

For more information about all arguments accepted by the CLI, have a look at the
[Arguments Section](#arguments)

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
|**-f, --format**|_String_|Format to use for saving. Default to the extension format if not provided|

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

List of all available encoding format:

* `png`
* `jpg`
* `tga`
* `tif`
* `pnm`
* `ico`
* `bmp`

Those formats can be used directly on the CLI using the `manual` command, or via
a configuration file (for `session` run).

## Library usage

### Swizzle

To swizzle an image, you need to create descriptors, which are structure containing
image to read + channel to extract.

Descriptors can be created using a `String` to which the channel is appended,
from a path, or even directly from a loaded image:

```rust
use swizzler::{ChannelDescriptor};

// From a string.
let descriptor = ChannelDescriptor::from_description("./my_input.png:0").unwrap();

// From path + channel
let path = std::Path::PathBuf::from("./my_input.png");
let descriptor = ChannelDescriptor::from_path(path, 0).unwrap();

// From an image + channel
let descriptor = ChannelDescriptor::from_path(my_image, 0).unwrap();

```

You can then use any of the following to crete a swizzled image:

* `to_luma()` ⟶ swizzle inputs into a _Grayscale_ image
* `to_lumaA()` ⟶ swizzle inputs into a _Grayscale-Alpha_ image
* `to_rgb()` ⟶ swizzle inputs into a _RGB_ image
* `to_rgba()` ⟶ swizzle inputs into a _RGBA_ image

Example:

```rust
use swizzler::{to_rgba};

let r_channel = ChannelDescriptor::from_path(..., ...).unwrap();
let a_channel = ChannelDescriptor::from_path(..., ...).unwrap();

let result = to_rgba(Ok(r_channel), None, None, Ok(a_channel)).unwrap();
```

> NOTE: you can use `None` to let a channel empty.

The result image is an `ImageBuffer` from the [image crate](https://docs.rs/image/0.23.2/image/struct.ImageBuffer.html). You can then manipulate it like any other image, e.g:

```rust
result.save("./output.png").unwrap();
```


### Running a session

