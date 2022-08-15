<img src="logo/logo.png" width="300px" alt="logo"/> &nbsp;
===
A quasi-lossless Balkanoidal meta-lingual compressor.

[![Crates.io](https://img.shields.io/crates/v/serbzip?style=flat-square&logo=rust)](https://crates.io/crates/serbzip)
[![docs.rs](https://img.shields.io/badge/docs.rs-serbzip_core-blue?style=flat-square&logo=docs.rs)](https://docs.rs/serbzip-core)
[![Build Status](https://img.shields.io/github/workflow/status/ekoutanov/serbzip/Cargo%20build?style=flat-square&logo=github)](https://github.com/ekoutanov/serbzip/actions/workflows/master.yml)
[![codecov](https://img.shields.io/codecov/c/github/ekoutanov/serbzip/master?style=flat-square&logo=codecov)](https://codecov.io/gh/ekoutanov/serbzip)

# Background
It has long been accepted that Serbian is a compact variant of Russian, with less liberal use of vowels. Since the forfeiting of the Riviera in 1991, the loss of tourism revenue has led to further austerity in vowel use. Serbs increasingly needed economically viable ways of communicating, since vowels aren't exactly cheap!

`serb.zip` is a lightweight framework for transcoding text from one lexical form to another. It presently comes with one codec — **Balkanoid** — a universal transcoder between Serbo-Croatian and Russian languages that is almost entirely isomorphic — it maps from one lingual domain to another with no loss of meaning and some loss of whitespace and capitalisation. Balkanoid works with both English and East-Slavic texts.

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

Let's key in the first verse of Mearns' _Antigonish_. Enter the following text, line by line, and hit CTRL+D when done.

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

To later use `custom.blk` during compression and expansion:

```sh
serbzip -d custom.blk ...
```

**CAUTION**: The congruency of dictionaries is essential for the reversibility of the algorithm. In other words, if you use one dictionary to compress a stream, then another dictionary to expand it, some words may not match. The default dictionary has ~50K English and ~50K Russian words, and should suffice for most people.

## Switching codecs
`serb.zip` encompasses several codecs, of which `balkanoid` is the default. Other codecs are still in development. Use the `--codec` flag to specify an alternate codec.

## Getting help
If you are using this application, you should probably seek professional help.

If you get stuck, run `serbzip --help`.

# Using `serb.zip` as a library
To embed `serb.zip` in your application, follow these [instructions](serbzip-core/README.md).

# How `serb.zip` Works
Each codec works differently. They share some common concepts but the underlying algorithms may be entirely different.

## Balkanoid
Balkanoid is the famed codec that started it all. It maps between compressed and expanded forms with _no loss in meaning_. It does this through a combination of _dictionary mapping_ and a _unary encoding_.

Balkanoid's dictionary is a mapping from a **fingerprint** to a sorted vector of all dictionary words that satisfy that fingerprint. A fingerprint is taken by stripping all vowels and converting the remaining characters to lowercase. E.g., the fingerprint of `"Apple"` is `"ppl"`.

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

Once the dictionary is loaded, the algorithm works by iterating over the input, line by line. Each line is subsequently tokenised by whitespace, disregarding contiguous whitespace (shown here with `␣` characters for readability) sequences. E.g., the line `"To␣be...␣␣␣␣␣or␣␣␣␣␣not␣to␣be!"` is captured as six words: `["To", "be...", "or", "not", "to", "be!"]`. An empty line or a line comprising only whitespace characters is tokenised to `[]`.

Each resulting word is subject to three rulesets — punctuation, compaction and capitalisation — applied in that order.

The **punctuation rules** are used to efficiently deal with punctuated words such as `"banana!"`. Normally, `"banana"` would be an easy dictionary lookup, but the exclamation mark is a fly in the ointment. Punctuation splits every word into a _prefix_ and a _suffix_. The prefix encompasses a contiguous string of alphabetical characters from the start of the word. The suffix contains all other characters.

1. If the first character of the word is a backslash (`'\'`), then move the backslash into the head of the prefix. For the 2nd and subsequent characters, move the longest contiguous string of alphabetical characters from the start of the string into the prefix, while assigning the rest of the string to the suffix.
2. If the first character is not a backslash, split the word such that the prefix encompasses the longest contiguous string of alphabetical characters from the start of the word. The suffix contains all other characters.

For example, for `"bananas!!!"`, the punctuation tuple is `("bananas", "!!!")`, by rule 2. For an unpunctuated word, the prefix encompasses the entire word (again, by rule 2), while the suffix is an empty string; e.g., `"pear"` splits to `("pear", "")`. For a word such as `"\foo?"`, the punctuation tuple is `("\foo", "?")`, by rule 1 — the first backslash is admitted to the prefix. For a single backslash `"\"`, the result is `("\", "")`, as per rule 1. For a series of backslashes `"\\\"`, the result is `("\", "\\")`.

When a word comprises multiple fragments separated by non-alphabetical characters, only the first fragment is assigned to the prefix; e.g., `"dog@pound.com"` becomes `("dog", "@pound.com")`. This is a rare example of a relatively long suffix; ordinarily, suffixes comprise trailing punctuation symbols, which are prevalent in English and East-Slavic languages. Although one might think that multi-fragment words could be expressed as N-tuples and encoded accordingly, this would render the algorithm irreversible for some words.

The **compaction rules** are used to de-vowel the word, being the essence of the algorithm. It starts by taking a lowercase representation of the prefix element of the punctuation tuple and removing all vowels (excluding `'y'` in the English variant). The compaction rule does not apply to the suffix element of the tuple — suffixes are encoded as-is. In practice, the suffix is much shorter than the prefix. The rule comprises four parts:

1. If the word-prefix begins with a backslash (`'\'`), then treat it as an escape sequence. Prepend another backslash and return the word-prefix. E.g., `"\sum"` encodes to `"\\sum"`. This rule is mainly used for encoding papers containing typesetting commands, such as TeX and Markdown.
2. Convert the word-prefix to lowercase and generate its fingerprint. Resolve the (0-based) position of the lowercase prefix in the vector of strings mapped from the fingerprint. If the word-prefix is in the dictionary, then encode it by padding the output with a string of whitespace characters — the length of the string equal to the position of the prefix in the vector — then output the fingerprint. E.g., assume the word-prefix `"no"` is positioned second in the dictionary mapping for its fingerprint: `"n" -> ["in", "no", "on"]`. It would then be encoded as `"␣n"` — one space followed by its fingerprint. The word `"on"` would have required two spaces — `"␣␣n"`.
3. Otherwise, if the word-prefix is not in the dictionary and contains one or more vowels, it is encoded as-is. E.g., the word-prefix `"tea"`, which is not in our sample dictionary, but contains a vowel, is encoded as `"tea"`.
4. Otherwise, if the word-prefix comprises only consonants and its fingerprint does not appear in the dictionary, it is encoded as-is. E.g., the word-prefix `"psst"` is not mapped, so it is encoded with no change — `"psst"`. This case is much more frequent in East-Slavic than it is in English; in the latter, words comprising only consonants are either abbreviations, acronyms or representations of sounds.
5. Otherwise, prepend a backslash (`'\'`) to the word-prefix. E.g., the word-prefix `"cnt"`, comprising all-consonants, having no resolvable position in `"cnt" -> ["cent", "count"]` for an existing fingerprint `"cnt"`, is encoded as `"\cnt"`. Without this rule, `"cnt"` with no leading spaces might be reversed into `"cent"`... or something entirely different and less appropriate.

The **capitalisation** rules encode capitalisation _hints_ into the output so that the word-prefix may have its capitalisation restored. These rules is not perfectly reversible, but cope well in the overwhelming majority of cases.

1. If the input contains a capital letter anywhere after the second letter, capitalise the entire input.
2. Otherwise, if the input starts with the capital letter, capitalise only the first letter and append the remainder of the input.
3. Otherwise, if the input has no capital letters, feed it through to the output unchanged.

The encoded output of each prefix is subsequently combined with the suffix to form the complete, encoded word.

Once the encoded outputs of each word have been derived, the resulting line is obtained by concatenating all outputs, using a single whitespace character to join successive pairs. The outputs from our earlier examples — `["␣n", "tea", "psst", "\cnt"]` are combined into the string `"␣n␣tea␣psst␣\cnt"`.

Like compression, expansion begins by tokenising each line. Only in the case of expansion, contiguous whitespace sequences are not discarded — their count is needed to decode the output of _compaction rule 1_. This is the _unary encoding_ part of the algorithm. One might think of it as the reverse of _run-length encoding_.

For each tokenised word, we apply the punctuation rules first, followed by reverse-compaction, then by capitalisation.

Punctuation here is the same as before. The word is split into a punctuation tuple. The prefix is decoded, while the suffix is carried as-is.

The **reverse-compaction** rule acts as follows:

1. If the word-prefix begins with a backslash, then it is removed, and the remaining substring is returned. E.g., the encoded word-prefix `"\trouble"` is decoded to `"trouble"`. A single backslash `"\"` decodes to an empty string. This rule reverses the output of both rule 1 and rule 5 of the compaction ruleset.
2. Otherwise, the lowercase version of the word-prefix is checked for vowels. If at least one vowel is found, the lowercase string is returned. E.g., the word-prefix `"german"` is passed through.
3. Otherwise, if the lowercase word-prefix comprises only consonants, it is considered to be a fingerprint. It is resolved in the dictionary by looking up the word at the position specified by the number of leading spaces. If the fingerprint is present in the dictionary, it should always be possible to locate the original word. (The exception is when two different dictionaries were used, which is treated as an error.) E.g., given the mapping `"n" -> ["in", "no", "on"]`, the encoded word-prefix `"␣␣n"` decodes to `"on"`.
4. Otherwise, if the fingerprint is not in the dictionary, return the word as-is. E.g., `"kgb"` is passed through if there is no mapping for its fingerprint in the dictionary.

After reverse-compaction, the capitalisation rule is applied as per the compression path. Capitalisation is mostly reversible — it works well for words that begin with capitals or contain only capitals, such as acronyms. However, it cannot always reverse mixed-case words and words that reduce to a single consonant. Consider some examples.

`"Apple"` encodes to `"Ppl`, given a dictionary containing `"ppl" -> ["apple", ...]`. It correctly decodes back to `"Apple"`.

`"KGB"` encodes to `"KGB"`, given no dictionary mapping for the fingerprint `"kgb"`, and decodes correctly.

`"LaTeX"` encodes to `"LTX"`, assuming it exists in the dictionary. It decodes to `"LATEX"`, incorrectly capitalising some letters. This is an example of the mixed-case problem. However, if the word is absent from the dictionary, it will be encoded as-is, and will have its capitalisation restored correctly.

The problematic mixed-case scenario seldom occurs in practice because acronyms are generally absent from dictionaries; such words are rarely subjected to compaction — they're encoded verbatim.

`"Ra"` (the sun god of ancient Egypt) is encoded to `"Ra"`, given a dictionary mapping `"r" -> ["ra", ...]`. Capitalisation is correctly reversed. However, if the input is the acronym `"RA"`, it will also be encoded to `"Ra"` — capitalisation will not be reversible in this case. Again, if `"ra"` is not in the dictionary, the word will be encoded verbatim and capitalisation will be reversible.

# Compression Efficacy
`serb.zip` is interesting from a linguistic standpoint, but does it actually _compress_?

The table below shows the result of compressing a series of literary works using the Balkanoid codec. The works ranged in size from a few hundred kilobytes to several megabytes. For each text, the size of the `serb.zip` output is displayed, along with a percent reduction relative to the original size. (Negative values indicate that the size increased relative to the original.)

Both the original text and its "serb-zipped" variant were subsequently subjected to `gzip` and `bzip2` binary compression on the `--best` setting. It was instructive to observe whether binary post-compression altered the result. The sizes were recorded, along with the reduction factor.

The complete test set was split into two: English texts and Russian texts. The English test set was much larger — 21 works versus 5.

## English texts
|filename                      |size      |words     |gzip size |bzip2 size|sz size   |sz reduction %|sz.gz size  |sz+gz reduction %|sz.bz2 size |sz+bz2 reduction %|
|-----------------------------:|---------:|---------:|---------:|---------:|---------:|-------------:|-----------:|----------------:|-----------:|-----------------:|
|       no_man_is_an_island.txt|       396|        81|       289|       283|       366|          7.57 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|         258|            10.72 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|         261|              7.77 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|                antigonish.txt|       478|        97|       282|       293|       552|        -15.48 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|         263|             6.73 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|         269|              8.19 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|    a_dream_within_a_dream.txt|       652|       141|       414|       404|       648|           .61 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|         381|             7.97 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|         374|              7.42 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|                 the_raven.txt|      6587|      1068|      2753|      2610|      6250|          5.11 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|        2513|             8.71 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|        2440|              6.51 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|             metamorphosis.txt|    142017|     25094|     51125|     41494|    135843|          4.34 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|       47295|             7.49 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|       40236|              3.03 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|       alice_in_wonderland.txt|    174313|     29594|     61021|     49027|    167360|          3.98 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|       57207|             6.25 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|       48216|              1.65 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|                the_prince.txt|    307808|     52982|    112051|     86353|    284168|          7.68 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      103035|             8.04 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|       84642|              1.98 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|        calculus_made_easy.txt|    404533|     56128|    128753|    103437|    376411|          6.95 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      122490|             4.86 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      102025|              1.36 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|              frankenstein.txt|    448821|     78122|    168673|    126241|    408328|          9.02 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      154384|             8.47 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      123996|              1.77 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|           sherlock_holmes.txt|    612668|     98533|    212198|    153006|    532337|         13.11 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      192828|             9.12 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      151134|              1.22 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|       pride_and_prejudice.txt|    798774|    124753|    267241|    182367|    671468|         15.93 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      241914|             9.47 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      182294|               .04 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|           effective_kafka.txt|    832006|    121588|    260631|    186466|    740534|         10.99 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      244580|             6.15 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      185031|               .76 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|                   dracula.txt|    881217|    164382|    335208|    245421|    855606|          2.90 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      315240|             5.95 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      244016|               .57 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|             new_testament.txt|   1020010|    182644|    344546|    248323|    978194|          4.09 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      326802|             5.14 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      247227|               .44 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|                 jane_eyre.txt|   1084733|    188452|    428029|    317669|   1028088|          5.22 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      397902|             7.03 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      311810|              1.84 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|  crime_and_punishment_eng.txt|   1201520|    206553|    439388|    319534|   1161086|          3.36 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      412300|             6.16 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      315694|              1.20 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|                 moby_dick.txt|   1276235|    215864|    511491|    389164|   1208689|          5.29 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      476706|             6.80 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      383487|              1.45 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|                    mormon.txt|   1588965|    293164|    453627|    313667|   1528402|          3.81 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      425963|             6.09 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      313170|               .15 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|         anna_karenina_eng.txt|   2068079|    352857|    743362|    535118|   1958820|          5.28 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      694156|             6.61 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      529153|              1.11 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|     count_of_monte_cristo.txt|   2786940|    464031|   1012102|    724410|   2600665|          6.68 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      939973|             7.12 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      714182|              1.41 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|
|         war_and_peace_eng.txt|   3359372|    566334|   1221693|    888312|   3120825|          7.10 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|     1134553|             7.13 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|      880502|               .87 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|

Within the English test set, the size reduction using `serb.zip` alone was substantial in most cases. The greatest reduction was seen in _Pride and Prejudice_ — 15.93%, followed by 13.11% in _The Memoirs of Sherlock Holmes_. In most cases, the size reduction was in the single-digit percentages. _A Dream Within a Dream_ showed almost no difference in size, with a reduction of just .61%.

However, in one case — _Antigonish_ — the output size increased by 15.48%.

Applying `gzip` and `bzip2` was where things got really interesting. In every single case, the output size decreased substantially, compared to the equivalent binary compression run without `serb.zip`. For `gzip`, improvements ranged from 4.86% (_Calculus Made Easy_) to 10.72% (_No Man is an Island_). For `bzip2`, the smallest improvement was recorded for _Pride and Prejudice_, at .04%; the highest improvement was for Antigonish, at 8.19%. This is interesting because _Antigonish_ showed a poor result for `serb.zip` alone. However, it appears that although the file size increased, the information entropy decreased at a disproportionately greater rate. This helped binary compressors achieve a better overall result.

## Russian texts
|filename                      |size      |words     |gzip size |bzip2 size|sz size   |sz reduction %|sz.gz size  |sz+gz reduction %|sz.bz2 size |sz+bz2 reduction %|
|-----------------------------:|---------:|---------:|---------:|---------:|---------:|-------------:|-----------:|----------------:|-----------:|-----------------:|
|               u_lukomorya.txt|      1599|       169|       754|       609|      1528|           4.44 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|         744|             1.32 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|         620|             -1.80 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|
|             lyublyu_tebya.txt|      2290|       205|      1032|       830|      2189|           4.41 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|        1022|              .96 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|         832|              -.24 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|
|                  odnazhdy.txt|      2359|       237|      1031|       885|      2333|           1.10 ![](https://via.placeholder.com/12/00ff00/00ff00.png)|        1047|            -1.55 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|         905|             -2.25 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|
|         anna_karenina_rus.txt|   3072666|    288389|    802824|    520413|   3319009|          -8.01 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|      841072|            -4.76 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|      528303|             -1.51 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|
|         war_and_peace_rus.txt|   5368089|    494556|   1436738|    935792|   5618619|          -4.66 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|     1490941|            -3.77 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|      955191|             -2.07 ![](https://via.placeholder.com/12/ff0000/ff0000.png)|

For the Russian test set, the result was very different. The size difference for the `serb.zip` run varied between 4.44% and -8.01%. The result failed to improve with the use of binary post-compression. Notably, _Анна Каренина_ and _Война и Мир_ showed the worst results across the board. They were also the largest works in the test set.

It appears that Balkanoid is not particularly effective at compressing Russian texts. The following postulates a theory as to why.

In (modern) English, a noun doesn’t have any other forms but singular and plural. In Russian, nouns are declined — a process called «склонение» (declension), which gives the words different endings, in singular and in plural in six cases: именительный падеж (nominative), родительный падеж (genitive), дательный падеж (dative), винительный падеж (accusative), творительный падеж (instrumental), and предложный падеж (prepositional). In addition, Russian has genders: masculine, feminine, and neuter.

When encoding words in English, provided both the singular and plural forms of a word are in the dictionary, that word may be effectively compacted in every case. In Russian, words have numerous variations and the likelihood of locating a given word in the dictionary exactly as it appears in the input text is substantially lower. Dictionary-based compaction alone, in the absence of grammatical awareness, appears to be insufficient to achieve consistent and measurable compression gains — comparable to that observed with English texts.

# FAQ
### Does `serb.zip` work?
`serb.zip` (namely, the Balkanoid codec) was tested on numerous texts, including such literary masterpieces as _War and Peace_ and _Effective Kafka_, and was found to correctly compress and expand each word in every case. A total of 27 texts spanning over 4 million words were assessed. Some even included typesetting commands. `serb.zip` works.

### What are its limitations?
The caveats and limitations are explained in the [how `serb.zip` works](#how-serbzip-works) section. In summary:

* Contiguous spaces between words, as well as leading and trailing spaces will not be restored.
* Some mixed case words will not have their capitalisation restored correctly.
* Some acronyms containing one consonant will not be restored correctly.
* The algorithm requires that the same dictionary is used for compression and expansion. It cannot detect if a dictionary has changed between runs. In many cases, this will produce a legible expansion, albeit the output may not correspond to the original.

In every other respect, Balkanoid is a fully restorable codec. In other words, the compressed output can be accurately restored to match the original input upon expansion. The limitations above are, by and large, immaterial — they do not affect comprehension.

### What is the difference between `serb.zip` and Balkanoid?
`serb.zip` is a framework for developing, testing and executing codecs that map from one lexical form to another. Balkanoid is an implementation of a codec. When running the `serbzip` command without specifying a codec, Balkanoid will be used by default.

### I love this project. How can I contribute?
If you're interested in this sort of piss-take, please, do join in.

It would be nice to have more reversible, human-readable transformations. [Pig Latin](https://en.wikipedia.org/wiki/Pig_Latin) might be a good example. The current algorithm is simple but not reversible. Perhaps, with little alteration, one might make a fully reversible/lossless Pig Latin codec.

`serb.zip` is highly modular; you can plug in your own `serbzip::codecs::Codec` implementation, and away you go.
