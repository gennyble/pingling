```metadata
Title Meta
```

# # Heading 1
## ## Heading 2
### ### Heading 3
#### #### Heading 4

This is some regular text. It's a `<p>` tag! Oh, yeah, inline code block, look at that! Separate paragraphs by two lines breaks (\n\n).

What if you want to break a paragraph? Use a single break.
Like this! See how hard breaks are respected? Markdown lets you break and leave lines touching and it'll remove the hard-break. That doesn't work here.

Okay, cool, but what can you do? *Italics like `*this*`*. Also some **bold like `**this**`**. You can *italic and **bold** at the same time*. That was hard to implement.

Links? Link externally with double curly braces and just putting that good ol' link right in there, like this: `{{url}}`. Here's my website {{https://nyble.dev}}.
Do you want the link to have text that isn't just the URL? You gotta use reference links for that, like this: `{!Reference}`. Here's my website again {!Nyble Dot Dev}. Don't forget the `[Reference]: url` for that. It must stand on a line of it's own.

[Nyble Dot Dev]: https://nyble.dev

We have all of this text, but what about an image? An image! Ah yes! So, make a link of any kind. Have it stand as it's own paragraph. On the next line, no empty line in between, put a carrot `^` and then some text. That's the alt text. If you want an image to appear as an image and not just a link, you *must* add alt text.

{!Textual Image}
^ An image of text in the font Grenze. It says "This is an image!"

[Textual Image]: https://textual.bookcase.name?font=Grenze&c=247&fs=32&text=This+is+an+image%21&forceraw

Here's a full example:
```
{!Link Reference}
^ Image alt text

[Link Reference]: URL
```

To link to another article, use a link with the name of the file. You don't even need the path! Well... more on that in {interlinking}.