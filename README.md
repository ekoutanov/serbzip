<img src="logo/logo.png" width="300px" alt="logo"/> &nbsp;
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

Let's key in the first verse of Mearns' "Antigonish". Enter the following text, line by line, and hit CTRL+D when done.

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

Most of the time, though, you'll want to compress a file and have the output written to another file:

```sh
serbzip -c -i antigonish.txt -o antigonish.sz
```

This reads from `antigonish.txt` in the current directory and writes to `antigonish.sz`. By convention, we use `.sz` extensions on `serb.zip` files, but you may use any extension you like.

## Expanding/decompressing a file
The expansion operation is the logical reverse of compression, and uses much the same arguments. The only difference: the `-x` flag instead of `-c`.

```sh
serbzip -x -i antigonish.sz -o antigonish.txt
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
Balkanoid is the famed codec that started it all. It maps between compressed and expanded forms with _no loss in meaning_. It does this through a combination of _dictionary mapping_ and a _unary encoding_.

Balkanoid's dictionary is a mapping from a **fingerprint** to a sorted vector of all dictionary words that satisfy that fingerprint. A fingerprint is taken by stripping all vowels and converting remaining characters to lowercase. E.g., the fingerprint of `"Apple"` is `"ppl"`.

The dictionary is essentially a `HashMap<String, Vec<String>>`. The vector is sorted by comparing words by length (shortest first), then by lexicographical order.

For the sample word list

```
at
came
cent
come
count
in
inn
it
no
on
```

the resulting dictionary is

```
"cm" -> ["came", "come"]
"cnt" -> ["cent", "count"]
"n" -> ["in", "no", "on"]
"t" -> ["at", "it"]
"nn" -> ["inn"]
```

Once the dictionary is loaded, the algorithm works by iterating over the input, line by line. Each line is subsequently tokenised by whitespace, disregarding contiguous whitespace (shown here with `␣` characters for readability) sequences. E.g., the line `"To␣be...␣␣␣␣␣or␣␣␣␣␣not␣to␣be!"` is captured as six words: `["To", "be...", "or", "not", "to", "be!"]`. An empty line, or a line comprising only whitespace characters is tokenised to `[]`.

Each resulting word is subject to three rule-sets — punctuation, compaction and capitalisation — applied in that order.

The **punctuation rules** are used to efficiently deal with punctuated words such as `"banana!"`. Normally, `"banana"` would be an easy dictionary lookup, but the exclamation mark is a fly in the ointment. Punctuation splits every word into a _prefix_ and a _suffix_. The prefix encompasses a contiguous string of alphabetical characters from the start of the word. The suffix contains all other characters.

1. If the first character of the word is a backslash (`'\'`), then move the backslash into the head of the prefix. For the 2nd and subsequent characters, move the longest contiguous string of alphabetical characters from the start of the string into the prefix, while assigning the rest of the string to the suffix.
2. If the first character is not a backslash, split the word such that the prefix encompasses the longest contiguous string of alphabetical characters from the start of the word. The suffix contains all other characters.

For example, for `"bananas!!!"`, the punctuation tuple is `("bananas", "!!!")`, by rule 2. For an unpunctuated word, the prefix encompasses the entire word (again, by rule 2), while the suffix is an empty string; e.g., `"pear"` splits to `("pear", "")`. For a word such as `"\foo?"`, the punctuation tuple is `("\foo", "?")`, by rule 1 — the first backslash is admitted to the prefix. For a single backslash `"\"`, the result is `("\", "")`, as per rule 1. For a series of backslashes `"\\\"`, the result is `("\", "\\")`.

When a word comprises multiple fragments separated by non-alphabetical characters, only the first fragment is assigned to the prefix; e.g., `"dog@pound.com"` becomes `("dog", "@pound.com")`. This a rare example of a relatively long suffix; ordinarily, suffixes comprise trailing punctuation symbols, which are prevalent in English and East-Slavic languages. Although one might think that multi-fragment words could be expressed as N-tuples and encoded accordingly, this would render the algorithm irreversible for some words.

The **compaction rules** are used to de-vowel the word, being the essence of the algorithm. It starts by taking a lowercase representation of the prefix element of the punctuation tuple and removing all vowels (excluding `'y'` in the English variant). The compaction rule does not apply to the suffix element of the tuple — suffixes are encoded as-is. In practice, the suffix is much shorter than the prefix. The rule comprises four parts:

1. Convert the word-prefix to lowercase and generate its fingerprint. Resolve the (0-based) position of the lowercased prefix in the vector of strings mapped from the fingerprint. If the word-prefix is in the dictionary, then encode it by padding the output with a string of whitespace characters — the length of the string equal to the position of the prefix in the vector — then output the fingerprint. E.g., assume the word-prefix `"no"` is positioned second in the dictionary mapping for its fingerprint: `"n" -> ["in", "no", "on"]`. It would then be encoded as `"␣n"` — one space followed by its fingerprint. The word `"on"` would have required two spaces — `"␣␣n"`.
2. Otherwise, if the word-prefix is not in the dictionary and contains one or more vowels, it is encoded as-is. E.g., the word-prefix `"tea"`, which is not in our sample dictionary, but contains a vowel, is encoded as `"tea"`.
3. Otherwise, if the word-prefix comprises only consonants and its fingerprint does not appear in the dictionary, it is encoded as-is. E.g., the word-prefix `"psst"` is not mapped, so it is encoded with no change — `"psst"`. This case is much more frequent in East-Slavic than it is in English; in the latter, words comprising only consonants are either abbreviations, acronyms or representations of sounds.
4. Otherwise, prepend a backslash (`'\'`) to the word-prefix. E.g., the word-prefix `"cnt"`, comprising all-consonants, having no resolvable position in `"cnt" -> ["cent", "count"]` for an existing fingerprint `"cnt"`, is encoded as `"\cnt"`. Without this rule, `"cnt"` with no leading spaces might be reversed into `"cent"`... or something entirely different and less fortunate.

The **capitalisation** rules encode capitalisation _hints_ into the output so that the word-prefix may have its capitalisation restored. These rules is not perfectly reversible, but cope well in the overwhelming majority of cases.

1. If the input contains a capital letter anywhere after the second letter, capitalise the entire input.
2. Otherwise, if the input starts with the capital letter, capitalise only the first letter and append the remainder of the input.
3. Otherwise, if the input has no capital letters, feed it through to the output unchanged.

The encoded output of each prefix is subsequently combined with the suffix to form the complete encoded word.

Once the encoded outputs of each word have been derived, the resulting line is obtained by concatenating all outputs, using a single whitespace character to join successive pairs. The outputs from our earlier examples — `["␣n", "tea", "psst", "\cnt"]` are combined into the string `"␣n␣tea␣psst␣\cnt"`.

Like compression, expansion starts by tokenising each line. Only in the case of expansion, contiguous whitespace sequences are not discarded — we need their count to decode the output of _compaction rule 1_. This is the _unary encoding_ part of the algorithm. One might think of it as the reverse of _run-length encoding_.

For each tokenised word, we apply the punctuation rules first, followed by reverse-compaction, then by capitalisation.

Punctuation here is the same as before. The word is split into a punctuation tuple. The prefix is decoded, while the suffix is carried as-is.

The **reverse-compaction** rule

In **reverse-compaction**, we


