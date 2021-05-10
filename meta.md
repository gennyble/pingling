# Notes on the Garden

This markup language is inspired by the {!Oscean Markup}, {!GemText}, and of course {!Markdown}.

[Oscean Markup]: https://wiki.xxiivv.com/site/meta.html
[GemText]: https://gemini.circumlunar.space/docs/cheatsheet.gmi
[markdown]: https://commonmark.org/

# Title
## subtitle
### section
#### subsection

How many heading levels do you really need? I guess I decided it was four.

### Inline Elements

Here is some **bold** and *italic* text. We're doing inline block elements?
Show me an `inline code block` and maybe even an internal link, in this case
circular because it's going back to the {meta}. Absolute links will be
displayed differently. If I link to the homepage, like this:
{https://bookcase.name}, it should get square brackets. If I, for some reason,
want to name that link, I *MUST* use a reference link. Reference links in this
markdown-but-not langauge require a bang afte the opening brace. I could use
them to, for example, link to {!Myself on Twitter}. Currently you cannot rename
internal links.

[Myself on Twitter]: https://twitter.com/gennyble

This derivative-of-markdown is designed to be as readable as possible when it's
in-writing or un-processed, and to be easily parseable. The hardest thing, I
think, will be the internal linking. It hasn't been properly described yet,
so let's get to that.

### Internal Linking
It isn't *too* complex. Imagine this directory structure:
/digital/audio/formats/mp3
/digital/visual/formats/gif

Now, you encounter a link to `{formats}`. Oh right, so internal links are
designed to be as short as possible, normally only requiring the final name
and none of the path. Anyway, `{formats}` is ambiguous; is it linking the audio
or visual formats? Well, that's exactly what you have to specify. Just add the
parent to the link so it looks like `{audio/formats}` and you're good!

What does that require of the parser? Well, I'd probably create a tree of the
garden in memory, but you could, in theory, look through it on every internal
link. I mean, I wouldn't recommend it, but if you're really tight on RAM, it
could be neccesary.

### Block Elements
Except for headings, there are a few block level elements that are useful and
I would not leave home without.

```langauge
Honestly, give me a monospaced code block any day.
It's so great
and
preserves
line breaks
```

What if you need to talk to the parser becaues it'd do something to the text that you'd rather it not?