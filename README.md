<img src="logo/logo.png" width="160px" alt="logo"/> `serb.zip`
===

# Background
It has long been accepted that Serbian is a compact variant of Russian, with less liberal use of vowels. Since the forfeiting of the Riviera in 1991, the loss of tourism revenue has led to further austerity in vowel use.

`serb.zip` is a universal transcoder between Serbo-Croatian and Russian languages that is almost entirely isomorphic — it maps from one lingual domain to another with no loss of meaning and some loss of whitespace and capitalisation. `serb.zip` works with both English and East-Slavic texts.

# Getting Started
## Installation
Install `serb.zip` via `cargo`:

```sh
cargo install serbzip
```

Unless a custom dictionary file is provided, `serb.zip` requires a compiled dictionary file in `~/.serbzip/dict.blk`. A default dictionary supporting English and Russian will be automatically downloaded and installed on first use.

## Compressing a file
When working with input/output, `serb.zip` can be given files to read and write from, or it can be run interactively. Let's try compressing an input stream interactively first.

```sh
serbzip -c
```

This brings up `serb.zip`, running the `balkanoid` codec, in interactive mode. 

```
██████╗  █████╗ ██╗     ██╗  ██╗ █████╗ ███╗   ██╗ ██████╗ ██╗██████╗
██╔══██╗██╔══██╗██║     ██║ ██╔╝██╔══██╗████╗  ██║██╔═══██╗██║██╔══██╗
██████╔╝███████║██║     █████╔╝ ███████║██╔██╗ ██║██║   ██║██║██║  ██║
██╔══██╗██╔══██║██║     ██╔═██╗ ██╔══██║██║╚██╗██║██║   ██║██║██║  ██║
██████╔╝██║  ██║███████╗██║  ██╗██║  ██║██║ ╚████║╚██████╔╝██║██████╔╝
╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═════╝

Enter text; CTRL+D when done.
```

Let's key in the first verse of Mearns' "Antagonish". Enter the following text, line by line, and hit CTRL+D when done.

```
Yesterday, upon the stair,
I met a man who wasn't there!
He wasn't there again today,
Oh how I wish he'd go away!
```

`serb.zip` echoes the compressed version:

```
Ystrdy,          pn th       str,
I  mt a mn wh wasn't      thr!
   H wasn't      thr         gn   tdy,
      H  hw I  wsh    h'd  g  wy!
```

Most of the time though, you'll want to compress a file and have the output written to another file:

```sh
serbzip -c -i antigonish.txt -o antagonish.sz
```

This reads from `antigonish.txt` in the current directory and writes to `antagonish.sz`. By convention, we use `.sz` extensions on `serb.zip` files, but you may use any extension you like.

## Expanding/decompressing a file
The expansion operation is the exact opposite of compression, and uses much the same arguments. The only difference: the `-x` flag instead of `-c`.

```sh
serbzip -x -i antigonish.sz -o antagonish.txt
```

# Advanced Usage
## Custom dictionary
`serb.zip` can be run with a custom dictionary file. The dictionary can be specified with the `-d` flag – either as a newline-separated word list (`.txt`) or a binary image (`.blk`). 

The word list is the simplest approach, but it takes time to parse the list, produce fingerprints and sort the mapping vectors. It is, nonetheless, useful for testing.

The preferred approach is to compile a binary image from a word list, and use the binary image thereafter.

To compile `custom.txt` into `custom.blk`:

```sh
serbzip -p -d custom.txt -m custom.blk
```

To later use `custom.blk` during compression/expansion:

```sh
serbzip -d custom.blk ...
```

**CAUTION**: The congruency of dictionaries is essential for the reversibility of the algorithm. In other words, if you use one dictionary to compress a stream, then another dictionary to expand it, some words may not match. The default dictionary has ~50K English and ~50K Russian words, and should suffice for most people.

## Switching codecs
`serb.zip` encompasses several codecs, of which `balkanoid` is the default. Other codecs are still in development. Use the `--codec` flag to specify an alternate codec.

## Getting help
If you are using this application, you should probably seek professional help.

If you get stuck, run `serbzip --help`.

# How `serb.zip` Works
Each codec works differently. They share some common concepts but the underlying algorithms may be totally different.

## Balkanoid
Balkanoid is codec that started it all. It maps between compressed and expanded forms with _no loss in meaning_.