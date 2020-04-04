<h1 align="center">Swizzler</h1>

![Swizzler Demo](/images/cli.gif)

<p style="text-align: center">Thanks to the <a href="https://freepbr.com/about-free-pbr/">Free PBR</a> website for the textures used in this demo</p>

## Installation

### 1. Download CLI

If you just need the **CLI**, you can download it directly in the [Release Tab](https://github.com/albedo-engine/swizzler/releases).

#### 2. Install CLI from sources

Alternatively, you can download, build, and install the _CLI_ locally using:

```sh
$ cargo install --git https://github.com/albedo-engine/swizzler.git
```

Check that the installation was successful:

```sh
$ swizzler --version
swizzler-cli 0.1.0
```

#### 3. Install library as a dependency

**Swizzler!** can also be used programmatically. Simply add a dependency to **Swizzler!** in your `Cargo.toml`:

```toml
[dependencies]
swizzler = { git = "https://github.com/albedo-engine/swizzler.git" }
```
## CLI Usage

### Manual

You can manually generate a new texture by providing the channel to extract for each source:

```sh
$ swizzler manual -i ./source_1.png:0 -i ./source_2.png:0 ...
```

Each `-i` argument takes a source image followed by the delimiting  character `:` and the channel to read.

The position of each `-i` argument is used to select the destination channel.

For instance, if you have an _RGB_ source image (`source.png`), and you want to shuffle the channels as _BGR_, you simply need to run:

```sh
$ swizzler manual -i ./source.png:2 -i ./source.png:1 -i ./source.png:0
```

The number of arguments defines the number of channels of the output image. For instance, generating a _grayscale_ image is done by using:

```sh
$ swizzler manual -i ./source.png:0
```

You can leave some channels empty by specifying the `none` keyword for an input:

```sh
$ swizzler manual -i red.png:0 -i none -i none -i alpha.png:3
```

### Folder processing

You may want to process a folder containing several textures. The [Manual Command](#manual)
is handy but can be difficult to use when you need to find what files should be grouped together.

Let's see how you could process an entire folder of images. In this example, we
are going to generate a texture mixing the metalness in the `red` channel, and
the roughness in the `alpha` channel.

Let's assume we have some textures in a folder `./textures`:

```sh
$ ls ./textures
enemy_albedo.png    enemy_metalness.png enemy_roughness.png
hero_albedo.png     hero_metalness.png  hero_roughness.png
```

We can define a configuration file to process those textures:

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

*  `base` attribute is used to extract the name of the asset (here `"hero"` or `"enemy"`)
* `matchers` attribute is used to identify the type of textures. Each entry will
look for a particular match
* `targets` attributes is used to generate new textures, using the files
resolved by the `matchers`.

To learn more about each attribute, please take a look at the
[Configuration File section](#configuration-file).

We can now run the CLI on our `textures` folder:

```sh
$ swizzler session --folder ./textures --config ./config.json
```

Alternatively, you can provide the `config.json` file on STDIN:

```sh
$ cat ./config.json | swizzler session --folder ./textures
```

The results will be generated in the folder `__swizzler_build`:

```sh
$ ls ./__swizzler_build
enemy-metalness-roughness.png hero-metalness-roughness.png
```

As you can see, the CLI extracted two kind of assets (`hero` and `enemy`), and
generated two textures. Each generated texture contains the metalness and the
roughness swizzled together.

### Configuration File

```
{

  "base": String,

  "matchers": [

      { "id": String, "matcher": String },
      ...

  ],

  "targets": [

      {
          "name": String,

          "output_format": String,

          "inputs": [

              [ "metalness", 0 ],
              ...

          ]
      }

  ]
}
```

#### `base` attribute

The `base` attribute describes how to extract the name of the asset from a path.
This **has to be** a [Regular Expression](https://en.wikipedia.org/wiki/Regular_expression) with **one** capturing group.

Example:

```json
"base": "(.*)_.*"
```

Captures everything before the last `_` occurence.

#### `matchers` attribute

The `matchers` attribute provide a list of files to match under the same asset.

* `id` is used to identify mathched files
* `matcher` provides a regular expression checking input files for a match.

Example:

```json
"matchers": [
    { "id": "metalness", "matcher": "(?i)metal(ness)?" },
    { "id": "roughness", "matcher": "(?i)rough(ness)?" }
]
```

In this example, file containing _"metalness"_ will be assigned the **id** `'metalness'`,
and files containing _"roughness"_ will be assigned the **id** `'roughness'`.

#### `targets` attributes

The `targets` attribute makes use of the `matchers` list to generate a new texture.

* `name` gets appended to the `base` name of the asset
* `output_format` chooses the encoding format of the generated texture. Take a look
at the [encoding formats](#encoding-formats) for all available options.

Example:

```json
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
```

Here, this target configuration will create a texture with the name `'{base}-metalness-roughness.png'`, for each asset containing a match for a
`metalness` and `roughness` source.

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

#### Encoding formats

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

You can generate a new texture from those descriptors using:

* `to_luma()` ⟶ swizzle inputs into a _Grayscale_ image
* `to_luma_a()` ⟶ swizzle inputs into a _Grayscale-Alpha_ image
* `to_rgb()` ⟶ swizzle inputs into a _RGB_ image
* `to_rgba()` ⟶ swizzle inputs into a _RGBA_ image

Those functions use descriptors (`ChannelDescriptor`) to generate the final
texture.

There are several ways to create descriptors:

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

Example generating a _RGBA_ texture:

```rust
use swizzler::{to_rgba};

let r_channel = ChannelDescriptor::from_path(..., ...).unwrap();
let a_channel = ChannelDescriptor::from_path(..., ...).unwrap();

// Generates a RGBA image with two descriptors. The output image `green`
// and `blue` channels are left empty.
let result = to_rgba(Some(r_channel), None, None, Some(a_channel)).unwrap();
```

> NOTE: you can use `None` to leave a channel empty.

The result image is an `ImageBuffer` from the [image crate](https://docs.rs/image/0.23.2/image/struct.ImageBuffer.html), that you can manipulate like any other image:

```rust
result.save("./output.png").unwrap();
```

### Running a session

You can run a session programmatically by creating an `AssetReader` (A.K.A a "resolver"),
and a `Session`.

```rust
use regex::Regex;
use swizzler::session::{
    GenericAssetReader
    GenericTarget,
    RegexMatcher,
    Session,
};

// Creates a resolver and add matcher to it. Remember that matchers
// are used to group files together under a common asset.
let resolver = GenericAssetReader::new()
  .set_base(Regex::new("(.*)_.*").unwrap())
  .add_matcher(
    Box::new(RegexMatcher::new("metalness", Regex::new(r"(?i)metal(ness)?").unwrap()))
  )
  .add_matcher(
    Box::new(RegexMatcher::new("roughness", Regex::new(r"(?i)rough(ness)?").unwrap()))
  )

// Creates a target. Each target describes a texture to generate.
let metal_roughness_target = GenericTarget::new(vec![
  ("metalness", 0),
  None,
  None,
  ("roughness", 0),
])

// The `Session` will generate images using multiple threads, and save them
// to disk.
let session = Session::new()
  .set_output_folder(...)
  .set_max_threads_nb(...)
  .add_target(metal_roughness_target);

// Reads all assets on the main thread, using our assets reader.
let assets = match resolve_assets_dir(&command.folder, &resolver) {
  Some(list) => list,
  Err(error) => eprintln!("Error reading folder: {:?}", error),
};

// Goes through all assets, load all sources, swizzle the textures and save them
// to disk.
let errors = session.run(&assets);
for e in &errors {
    eprintln!("Error processing file: {:?}", e);
}
```

## Contributing

Contributions are welcome and appreciated!

This CLI has been written as a project to learn Rust. It's the first piece of
Rust code I've ever written, and it's likely that I made wrong design decisions.

If you have any ideas about how to improve the architecture or the performance, please
feel to contribute by raising an issue or creating a pull request.

When contributing to the library, please ensure that all the tests pass using:

```sh
$ cargo test
```

The library is formatted using [rustfmt](https://github.com/rust-lang/rustfmt).
You can run the formatter by using:

```sh
cargo fmt
```

## Notes

**Swizzler!** being my first Rust project, I needed a template source code for inspiration on best practices.

This CLI is heavily inspired by [Texture Synthesis](https://github.com/EmbarkStudios/texture-synthesis) from the EmbarkStudios team. Thanks for their open source
contributions!
